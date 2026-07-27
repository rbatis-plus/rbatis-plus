# MyBatis-Plus Enhance 架构与代码导读

> 本文档基于本地 **codegraph** 索引（`/Users/wandl/workspaces/workspace-github-easy-4-java/mybatis-plus-enhance`，105 个 Java 文件 + 7 xml + 2 properties + 1 yaml）的一手源码梳理。
>
> 仓库作者：**hiwepy**（与 MyBatis-Plus 无关，是第三方独立项目，对 baomidou 风格的扩展）。
>
> 参考资料：
> - GitHub：<https://github.com/hiwepy/mybatis-plus-enhance>
> - 实际工作区路径：`/Users/wandl/workspaces/workspace-github-easy-4-java/mybatis-plus-enhance`（不是 `/Users/wandl/workspaces/workspace-github/`，本工作区无此目录）
> - 当前分支：**`2.0.x`**（CI-Friendly 版本 `${revision}` = `2.0.x.20260630-SNAPSHOT`）
> - 技术栈基线：Java 17 / Spring 6.2.x / `jakarta.*` namespace / MyBatis 3.5.19 / MyBatis-Plus 3.5.14 / Hutool 5.8.40
>
> ---
>
> ## 在这里的原因
>
> 本文档路径：`rbatis-plus/docs/mybatis-plus-enhance/architecture.md`。
>
> 因为它是 **rbatis-plus Rust 移植** 的设计参考之一：
> - **本仓库** = MyBatis-Plus 之上的"企业级增强层"——加密/签名/数据权限/i18n/SQL 观测都是它给出 Java 路径
> - **MyBatis-Plus（父仓库 baomidou）** 提供框架；**MyBatis-Plus Enhance** 把企业级安全/审计诉求模块化
>
> 读这份文档可以回答：
>
> - Java 的"数据加密 + 签名 + 数据权限" 在拦截器链上**应该按什么顺序**执行？→ **§5 `EnhancePhase` 增强阶段顺序**
> - "查询结果解密 + 表签名"如何在 SQL 执行后回到**已映射的 POJO**？→ **§4 `MybatisPlusEnhanceInterceptor` 后置钩子**
> - 数据加密应该加在 `beforeUpdate` 之前还是之后？→ **§5 阶段顺序约定**
> - SQL 观测钩子应该怎么设计才能既支持 trace 又支持 slow SQL log？→ **§9 `SqlObservation` + `Sink` 抽象**
>
> 两份文档互相引用见末尾"相关阅读"。

---

## 目录

1. 一句话定位与设计哲学
2. 多模块仓库布局与规模
3. 关键概念：`EnhancePhase`（增强阶段顺序）
4. 拦截器入口：`MybatisPlusEnhanceInterceptor`
5. 拦截器 SPI：`EnhanceInnerInterceptor`
6. **八大增强拦截器**详解
7. 数据加密 + 解密（核心 pair）
8. 表签名 + 验签（核心 pair）
9. 数据权限（`DataScopePlus` 注解）
10. 多租户：`DefaultTenantLineHandler`
11. 实体国际化：`DataI18nInnerInterceptor`
12. SQL 观测：`SqlObservation` + 多个 `Sink`
13. SQL 注入器：`EnhanceSqlInjector` + 6 个方法文件
14. Mapper：`EnhanceBaseMapper` + `IEnhanceService` + `EnhanceServiceImpl`
15. 上下文对象：`TenantContext` / `SignatureUpdateContext` / `I18nContext` / `InsertIgnoreContext` / `SignatureVerificationContext`
16. 数据流 ASCII 流程图
17. 与 MyBatis / MyBatis-Plus / rbatis 对照速记表
18. 关键设计权衡（FAQ）
19. codegraph 速查命令
20. 推荐阅读顺序
21. 已废弃 / 注意点 / 关联注解
22. 与 rbatis-plus 移植 checklist

---

## 1. 一句话定位与设计哲学

**`mybatis-plus-enhance` = MyBatis-Plus 之上的 **企业级增强层**——不动官方的分页/乐观锁/多租户基础设施，补 8 个独立拦截器。**

> "在不替代官方插件体系的前提下，补充以下能力"——README

它做了三类事：

1. **安全与隐私**：字段级 AES 加密、HMAC、表级签名、验签
2. **多租户/数据权限/国际化**：`TenantLineInnerInterceptor`、`DataPermission` 注解驱动的行级过滤、`DataI18n`
3. **可观测性**：超长 SQL 检测、真实执行耗时观测、慢 SQL 日志、`ServiceLoader` 插件体系

### 1.1 三条版本线（README 表格节选）

| 项目版本线 | Java 基线 | Spring 栈 | Namespace | JSqlParser |
|---|---|---|---|---|
| `1.0.x` | JDK 8 | Spring Framework 5.3.x | `javax.*` | `mybatis-plus-jsqlparser-4.9` |
| **`2.0.x`**（当前） | JDK 17 | Spring Framework 6.2.x | `jakarta.*` | `mybatis-plus-jsqlparser` |
| `3.0.x` | JDK 21 | Spring 6.2.x | `jakarta.*` | `mybatis-plus-jsqlparser` |

——三版本平行演进，对应的源码分别存放在 `mybatis-plus-enhance-*-3.0.x/` 子目录。

### 1.2 模块拆分原则

README 明确：

> Spring 集成已隔离到 `mybatis-plus-enhance-spring`。`core` 与 `extension` 保持 Spring 无关；
> 普通 MyBatis-Plus 项目只需按能力选择 `core` 或 `extension`。

含义：**Core**（拦截器核心 + 阶段枚举）几乎不写业务——只放"Enhance SPI"；**Extension**（具体 8 个拦截器）才是企业级能力；**Spring** 提供 `@Transactional` Service 增强。

---

## 2. 多模块仓库布局与规模

```
mybatis-plus-enhance/                              Maven 多模块
├── pom.xml                                     parent (revision=2.0.x.20260630-SNAPSHOT)
├── README.md / COMPATIBILITY.md
├── mybatis-plus-enhance-core/                   ── ★ 核心 SPI（5 个 main 源 java）
│   └── src/main/java/com/baomidou/mybatisplus/enhance/
│       ├── util/ParameterUtils                  ── 通用"开关/参数"判断
│       ├── enums/BooleanEnum                    ── true/false 字符串识别
│       └── plugins/
│           ├── MybatisPlusEnhanceInterceptor    ── ★ 263 行：扩展官方 MybatisPlusInterceptor
│           └── inner/
│               ├── EnhanceInnerInterceptor      ── 91 行 SPI
│               └── EnhancePhase                 ── 46 行枚举（顺序校验）
├── mybatis-plus-enhance-extension/              ── ★ 60 个 main 源 java（占绝大多数）
│   └── src/main/java/com/baomidou/mybatisplus/enhance/
│       ├── crypto/                              ── 加解密 + 签名 + 验签
│       │   ├── handler/                              DataEncryptionHandler / DataSignatureHandler
│       │   ├── key/                                  CryptoKeyProvider SPI
│       │   └── enums/                                CipherPadding / CipherMode / HmacType / ...
│       ├── context/                             ── 5 个 ThreadLocal-based Context
│       ├── datascope/                           ── 数据权限
│       │   ├── handler/DataScopeAnnotationHandler
│       │   ├── handler/DataScopeExpressionProvider
│       │   └── toolkit/DataScopeExpressions
│       ├── tenant/DefaultTenantLineHandler
│       ├── i18n/                                ── 国际化
│       │   ├── handler/{DataI18nHandler, DefaultDataI18nHandler, DataInputProvider, LocaleProvider}
│       │   ├── context/I18nContext
│       │   ├── interceptor/DataI18nInnerInterceptor
│       │   └── bundle/{MultipleResourceBundle, EmptyResourceBundle, I18nListResourceBundle, KeyValuePair, ResourceBundleEnumeration}
│       ├── observation/                         ── SQL 观测
│       │   ├── SqlObservation / SqlObservationSink / SlowSqlLoggingSink
│       ├── plugins/inner/                       ── 8 个具体 InnerInterceptor
│       │   ├── DataEncryptionInnerInterceptor
│       │   ├── DataSignatureInnerInterceptor
│       │   ├── DataDecryptionInnerInterceptor
│       │   ├── DataI18nInnerInterceptor
│       │   ├── LongSqlInnerInterceptor
│       │   ├── InsertIgnoreInnerInterceptor
│       │   └── SqlObservationInnerInterceptor
│       ├── injector/methods/                    ── 6 个新 AbstractMethod 子类
│       │   ├── SelectIgnoreDecryptById / SelectIgnoreDecryptBatchIds
│       │   ├── SelectIgnoreDecryptList / SelectIgnoreDecryptMaps / SelectIgnoreDecryptObjs
│       │   └── UpdateSignatureById
│       ├── injector/EnhanceSqlInjector          ── 继承 DefaultSqlInjector
│       ├── mapper/EnhanceBaseMapper             ── 继承 BaseMapper，多 6 个 selectIgnoreDecrypt 方法
│       ├── result/                              ── 反射结果拷贝器（CRC/不拷贝）
│       └── util/{TableFieldHelper, EnhanceConstants, SymmetricCryptoUtil}
├── mybatis-plus-enhance-spring/                 ── 2 个 main 源 java
│   └── src/main/java/com/baomidou/mybatisplus/enhance/service/
│       ├── IEnhanceService
│       └── impl/EnhanceServiceImpl               ── 继承 MyBatis-Plus ServiceImpl + 加事务注解
└── mybatis-plus-enhance-alignment-test/         ── 对齐测试套件
```

codegraph 总览：

```
Files:     115   (Java 105, xml 7, properties 2, yaml 1)
Main java: 67 个
Total LOC: ~6977 行
```

---

## 3. 关键概念：`EnhancePhase`（增强阶段顺序）

`mybatis-plus-enhance-core/.../plugins/inner/EnhancePhase.java`，46 行，**整个框架的密钥**：

```java
public enum EnhancePhase {
    SQL_REWRITE(100),             // SQL 结构改写或前置保护
    PARAMETER_ENCRYPTION(200),   // 写入参数加密
    DATA_SIGNATURE(300),         // 写入签名 + 查询结果验签
    RESULT_DECRYPTION(400),      // 查询结果解密
    RESULT_I18N(500),            // 查询结果国际化
    OBSERVATION(900),            // SQL 执行观测与旁路通知
    UNSPECIFIED(Integer.MIN_VALUE);  // 不参与强制排序（自定义阶段）

    private final int order;
    EnhancePhase(int order) { this.order = order; }
    public int getOrder() { return order; }
}
```

### 3.1 设计要点

- **数值越小越先执行**：所有具体拦截器都返回 `phase()`，框架强制检查
- **`UNSPECIFIED`** 留口给第三方自定义（保留与官方 `InnerInterceptor` 接口的兼容性）
- 顺序含义在源码注释里写得很清楚：
  > "参数先加密再签名，查询结果先验签再解密，解密后才能执行国际化，观测通知最后执行"

### 3.2 验证逻辑（`MybatisPlusEnhanceInterceptor.addInnerInterceptor`）

```java
@Override
public void addInnerInterceptor(InnerInterceptor innerInterceptor) {
    Objects.requireNonNull(innerInterceptor, "innerInterceptor must not be null");
    List<InnerInterceptor> candidate = new ArrayList<>(getInterceptors());
    candidate.add(innerInterceptor);
    validateEnhanceOrder(candidate);
    super.addInnerInterceptor(innerInterceptor);
}

private void validateEnhanceOrder(List<InnerInterceptor> interceptors) {
    EnhancePhase previousPhase = null;
    Class<?> previousType = null;
    for (InnerInterceptor interceptor : interceptors) {
        if (!(interceptor instanceof EnhanceInnerInterceptor)) continue;
        EnhancePhase phase = ((EnhanceInnerInterceptor) interceptor).phase();
        if (phase == EnhancePhase.UNSPECIFIED) continue;     // 跳过自定义
        if (Objects.nonNull(previousPhase) && phase.getOrder() < previousPhase.getOrder()) {
            throw new IllegalArgumentException("Invalid enhance interceptor order: "
                + interceptor.getClass().getName() + " [" + phase + "] must not run after "
                + previousType.getName() + " [" + previousPhase + "]");
        }
        previousPhase = phase;
        previousType = interceptor.getClass();
    }
}
```

——一旦注册顺序违反 `EnhancePhase.order`，启动即抛异常。**这是设计中最值得学的"靠顺序约束来表达业务不变量"的写法**。

---

## 4. 拦截器入口：`MybatisPlusEnhanceInterceptor`

`mybatis-plus-enhance-core/.../plugins/MybatisPlusEnhanceInterceptor.java`，263 行。**继承官方 MybatisPlusInterceptor 在前两阶段（pre 钩子）复用官方语义，自己接管 query 与 update 的执行**——才能在执行完成时调用 `afterQuery / afterUpdate / afterExecution`。

### 4.1 `@Intercepts` 与官方一致

```java
@Intercepts({
    @Signature(type = StatementHandler.class, method = "prepare",    args = {Connection.class, Integer.class}),
    @Signature(type = StatementHandler.class, method = "getBoundSql", args = {}),
    @Signature(type = Executor.class,         method = "update",      args = {MappedStatement.class, Object.class}),
    @Signature(type = Executor.class,         method = "query",      args = {MappedStatement.class, Object.class, RowBounds.class, ResultHandler.class}),
    @Signature(type = Executor.class,         method = "query",      args = {MappedStatement.class, Object.class, RowBounds.class, ResultHandler.class, CacheKey.class, BoundSql.class})
})
@Slf4j
public class MybatisPlusEnhanceInterceptor extends MybatisPlusInterceptor { ... }
```

### 4.2 调度的 4 个阶段（核心 263 行可读为 4 段）

#### 4.2.1 SELECT 分支：自己执行 query

```java
if (!isUpdate && ms.getSqlCommandType() == SqlCommandType.SELECT) {
    ...
    for (InnerInterceptor interceptor : super.getInterceptors()) {
        if (!interceptor.willDoQuery(executor, ms, parameter, rowBounds, resultHandler, boundSql)) {
            return Collections.emptyList();
        }
        interceptor.beforeQuery(executor, ms, parameter, rowBounds, resultHandler, boundSql);
    }
    return executeQuery(executor, ms, parameter, rowBounds, resultHandler, boundSql);
}
```

#### 4.2.2 UPDATE 分支：自己执行 update

```java
} else if (isUpdate) {
    for (InnerInterceptor update : super.getInterceptors()) {
        if (!update.willDoUpdate(executor, ms, parameter)) return -1;
        update.beforeUpdate(executor, ms, parameter);
    }
    BoundSql boundSql = ms.getBoundSql(parameter);
    return executeUpdate(invocation, executor, ms, parameter, boundSql);
}
```

#### 4.2.3 **新增 `executeQuery`：执行 → afterQuery → afterExecution**

```java
private Object executeQuery(Executor executor, MappedStatement ms, Object parameter, RowBounds rowBounds,
                            ResultHandler<?> resultHandler, BoundSql boundSql) throws Throwable {
    CacheKey cacheKey = executor.createCacheKey(ms, parameter, rowBounds, boundSql);
    List<Object> result = null;
    Throwable failure = null;
    long startedAt = System.nanoTime();
    long elapsedNanos = 0L;
    try {
        result = executor.query(ms, parameter, rowBounds, resultHandler, cacheKey, boundSql);
        elapsedNanos = System.nanoTime() - startedAt;
        for (InnerInterceptor interceptor : super.getInterceptors()) {
            if (interceptor instanceof EnhanceInnerInterceptor) {
                EnhanceInnerInterceptor enhanceInterceptor = (EnhanceInnerInterceptor) interceptor;
                result = Objects.requireNonNull(
                        enhanceInterceptor.afterQuery(executor, ms, parameter, rowBounds, resultHandler, boundSql, result),
                        () -> enhanceInterceptor.getClass().getName() + " returned a null query result");
            }
        }
        return result;
    } catch (Throwable throwable) {
        failure = throwable;
        throw throwable;
    } finally {
        if (elapsedNanos == 0L) elapsedNanos = System.nanoTime() - startedAt;
        notifyAfterExecution(executor, ms, parameter, boundSql, result, failure, elapsedNanos);
    }
}
```

——这是与 `MybatisPlusInterceptor` 的**最大差异**：官方版直接调 `invocation.proceed()`（即 `executor.query(...)`），而本拦截器自执行后能取到 **`List<Object> result`** 并广播给所有 `EnhanceInnerInterceptor.afterQuery(...)`。

#### 4.2.4 **新增 `notifyAfterExecution`：广播到所有后置钩子**

```java
private void notifyAfterExecution(Executor executor, MappedStatement ms, Object parameter, BoundSql boundSql,
                                  Object result, Throwable failure, long elapsedNanos) {
    for (InnerInterceptor interceptor : super.getInterceptors()) {
        if (!(interceptor instanceof EnhanceInnerInterceptor)) continue;
        try {
            ((EnhanceInnerInterceptor) interceptor)
                .afterExecution(executor, ms, parameter, boundSql, result, failure, elapsedNanos);
        } catch (RuntimeException exception) {
            log.warn("Enhance after-execution listener failed: {}",
                interceptor.getClass().getName(), exception);
        }
    }
}
```

——**单点超时 / 单点失败不波及其他监听器**。这是从 `MybatisPlusInterceptor` 之后又一个吸收实战教训的设计。

---

## 5. 拦截器 SPI：`EnhanceInnerInterceptor`

`mybatis-plus-enhance-core/.../plugins/inner/EnhanceInnerInterceptor.java`，91 行：

```java
public interface EnhanceInnerInterceptor extends InnerInterceptor {

    default EnhancePhase phase() { return EnhancePhase.UNSPECIFIED; }

    default List<Object> afterQuery(Executor executor, MappedStatement ms, Object parameter, RowBounds rowBounds,
                                    ResultHandler<?> resultHandler, BoundSql boundSql,
                                    List<Object> rtList) throws SQLException {
        return rtList;
    }

    default void afterUpdate(Executor executor, MappedStatement ms, Object parameter, BoundSql boundSql,
                             int affectedRows) throws SQLException {
        // do nothing
    }

    default void afterExecution(Executor executor, MappedStatement ms, Object parameter, BoundSql boundSql,
                                Object result, Throwable failure, long elapsedNanos) {
        // do nothing
    }
}
```

——继承官方 `InnerInterceptor`（保留 `beforeQuery / beforeUpdate / beforePrepare / beforeGetBoundSql`），新增三个 **after 钩子**。

每个 `EnhanceInnerInterceptor` 子类都通过 `phase()` 声明自己的阶段，**框架强制按阶段顺序串起来调用**。

---

## 6. 八大增强拦截器详解

| 拦截器 | 文件 | 阶段 | 关键作用 |
|---|---|---|---|
| `DataEncryptionInnerInterceptor` | `plugins/inner/` | `PARAMETER_ENCRYPTION(200)` | 写入/查询参数加密 |
| `DataSignatureInnerInterceptor` | `plugins/inner/` | `DATA_SIGNATURE(300)` | 写入签名 + 查询验签 |
| `DataDecryptionInnerInterceptor` | `plugins/inner/` | `RESULT_DECRYPTION(400)` | 查询结果解密 |
| `DataI18nInnerInterceptor` | `i18n/interceptor/` | `RESULT_I18N(500)` | 实体字段国际化映射 |
| `LongSqlInnerInterceptor` | `plugins/inner/` | `SQL_REWRITE(100)` | 超长 SQL 检测 |
| `InsertIgnoreInnerInterceptor` | `plugins/inner/` | `SQL_REWRITE(100)` | MySQL `INSERT IGNORE` 作用域 |
| `SqlObservationInnerInterceptor` | `observation/` | `OBSERVATION(900)` | SQL 观测 + 慢 SQL 日志 |

加上 `TenantLineHandler`（基于官方 `TenantLineHandler` 接口实现，不是拦截器）。

接下来挑最重要的两对详解。

---

## 7. 数据加密 + 解密（核心 pair）

### 7.1 `DataEncryptionInnerInterceptor`（216 行）

**关键点**：
- 继承 `JsqlParserSupport` —— 可以选择性地用 jsqlparser 解析 SQL 找字段（框架不强制）
- `beforeQuery` / `beforeUpdate` 各检查一次 `@IgnoreEncrypted` 注解：
  - 用 `ms.getId()` 反查 mapper class + method → `Method.getAnnotation(IgnoreEncrypted.class)`
  - 标注时跳过整段加密
- 支持三种参数结构：
  - 直接的 entity 参数
  - `Map { Constants.ENTITY, entity }` → MyBatis-Plus 写法
  - `Map { Constants.WRAPPER, wrapper }` 且 wrapper 是 `Update` 类型 → UpdateWrapper
- 参数去重 `new HashSet<>(paramMap.values())` 防止同一 entity 在多 key 下被反复加密

### 7.2 `DataDecryptionInnerInterceptor`

与加密对称的 **查询后置** 钩子（`phase() = RESULT_DECRYPTION`），只在 `afterQuery` 中执行 —— **从来不在 before 阶段**。这是顺序约束的精髓：先验签、后解密。

### 7.3 字段级算法 `EncryptedFieldHandler` SPI

`crypto/handler/EncryptedFieldHandler.java` + `DefaultEncryptedFieldHandler.java` 是单字段加解密接口，框架可以让你：

- 自定义字段级算法（AES、SM4、AES-GCM、chacha20）
- 对不同字段用不同算法
- 提供 key 索引

`CryptoKeyProvider`（`crypto/key/`）把 key 从 `CryptoKeyMaterial` 静态提供或动态派生。

---

## 8. 表签名 + 验签（核心 pair）

### 8.1 `DataSignatureInnerInterceptor`（253 行）

比加密更复杂一些，**自带 `SignatureUpdateStrategy` 上下文**：

```java
@Override
public EnhancePhase phase() { return EnhancePhase.DATA_SIGNATURE; }
```

阶段职责在源码注释里写得很清楚：

> "写入前根据参数生成签名，查询后可选地验证结果完整性。若签名覆盖加密后的字段，写入顺序必须是 DataEncryptionInnerInterceptor 后接本拦截器；读取顺序必须先验签、再由 DataDecryptionInnerInterceptor 解密。签名和验签开关相互独立，便于渐进式迁移历史数据。"

**`beforeUpdate`** 检查 `SignatureUpdateContext.current()`：

| 策略 | 行为 |
|---|---|
| `DEFAULT` | 按 entity 计算签名 |
| `DEFERRED_RESIGN` | 跳过（已签名状态不变） |
| `SIGNATURE_ONLY` | 不签行（只更新签名列） |
| `REJECT_PARTIAL` | 拒绝签名表的部分更新 → 必须 `FULL_ROW` 整行签 |

**`afterQuery`** 验签（带 `signVerify` 开关）：

```java
for (Object rawObject : rtList) {
    if (Objects.isNull(rawObject) || SimpleTypeRegistry.isSimpleType(rawObject.getClass())) continue;
    getDataSignatureHandler().doSignatureVerification(rawObject, rawObject.getClass());
}
```

**注意**：渐进式迁移历史数据时 — 把 `signVerify` 开 → 历史数据无法验签→**关** 验签开关；把 `signSwitch` 关 → 只对新增数据签 ；`signSwitch + signVerify = (true, true)` 时全表必须已签。

### 8.2 `ResolveEntityClass` 解析签名表对应的实体类型

`resolveEntityClass(mappedStatement, parameterObject)` 三层回退：

1. `parameterObject instanceof Map` 且包含 `Constants.ENTITY` → 取 entity
2. `Map.containsKey(Constants.WRAPPER)` 且 wrapper 是 `AbstractWrapper` → 取 wrapper 内部 entity class
3. 上述都失败 → 用 statement id 解析 mapper 类，通过 `GenericTypeUtils.resolveTypeArguments(mapperClass, BaseMapper.class)` 反查泛型 T

——这是 MyBatis-Plus 的标准手法，签名拦截器沿用之。

---

## 9. 数据权限（`DataScopePlus` 注解）

数据权限不在主仓（baomidou）main，而是单独包，独立 InnerInterceptor + 注解。

### 9.1 注解

`datascope/annotation/DataScopePlus.java`：

```java
@DataScopePlus(
    resolveClass = MyDataScopeResolve.class,   // 自定义 resolver
    includeTables = {"t_user", "t_role"},    // 限制生效范围（可选）
    excludeTables = {"t_log"}                  // 或排除（可选）
)
public interface UserService {
    List<User> listAll();     // 调用时校验权限行级过滤
}
```

### 9.2 三个 handler

- `DataScopeAnnotationHandler` 负责扫描 `@DataScopePlus`，注入 `resolveClass` 实例
- `DataScopeExpressionProvider` 是 SPI——业务方实现的"行级 SQL 拼接器"
- `DataScopeExpressions` 工具类，把 `ExpressionProvider.getExpression(...)` 压回 JSqlParser 的 AST

——本质是"ORM 反射 + JSqlParser 改写 SQL where 子句"。

> **注释限制**：README 写"不会重新实现分页、乐观锁、多租户或数据权限等官方已有基础设施"。所以本仓库数据权限仅提供"扩展点"而非开箱即用——必须自己实现 `resolveClass`。

---

## 10. 多租户：`DefaultTenantLineHandler`

`tenant/DefaultTenantLineHandler.java`，121 行。实现 MyBatis-Plus 官方 `TenantLineHandler`：

```java
public class DefaultTenantLineHandler implements TenantLineHandler {
    public static final String DEFAULT_TENANT_COLUMN = "tenant_id";
    private static final Predicate<String> NEVER_IGNORE = tableName -> false;

    private final TenantContext context;
    private final String tenantColumn;
    private final Predicate<String> ignoredTable;
    ...
    @Override
    public Expression getTenantId() {
        Object tenantId = context.getCurrentTenantId();
        if (Objects.isNull(tenantId)) throw new IllegalStateException("Tenant ID is missing from TenantContext");
        if (tenantId instanceof Number) return new LongValue(tenantId.toString());
        return new StringValue(tenantId.toString());
    }
}
```

`TenantContext`（`context/TenantContext.java`）是 thread-local 当前租户 ID 容器。

```java
public class TenantContext {
    private static final ThreadLocal<Object> CURRENT = new ThreadLocal<>();

    public void setCurrentTenantId(Object tenantId) { CURRENT.set(tenantId); }
    public Object getCurrentTenantId() { return CURRENT.get(); }
    public void clear() { CURRENT.remove(); }
}
```

——通常由 Spring Web 拦截器 / 网关层在 session / token 解出后填入，业务线程结束时清空。

---

## 11. 实体国际化：`DataI18nInnerInterceptor`

`i18n/interceptor/DataI18nInnerInterceptor.java` + `i18n/handler/` 系列。

### 11.1 包结构

- `DataI18nHandler` SPI——业务方实现国际化字典查询
- `DefaultDataI18nHandler` 默认实现，配合 bundle/ 目录
- `DataInputProvider` 提供"输入 key → 输出 value"的查找入口
- `LocaleProvider` 提供当前 locale（默认走 `Locale.getDefault()`）
- `bundle/` 5 个文件：多种 ResourceBundle 适配（JDK `PropertyResourceBundle` 自定义 + 列表式 bundle + 强制枚举）
- `context/I18nContext` Thread-local 当前 locale

### 11.2 工作流（典型）

`phase() = RESULT_I18N(500)` → **在 `afterQuery` 阶段**：MyBatis 已经把 ResultSet 映射为 POJO；扫描 entity 上需要国际化的字段；调 `DataI18nHandler.translate(fieldValue, locale)`；覆盖回去。

——这与加解密一样典型：**所有对结果行的"transformations"都要排在 SQL 执行之后**。

---

## 12. SQL 观测：`SqlObservation` + 多个 `Sink`

### 12.1 `SqlObservationInnerInterceptor`

```java
public class SqlObservationInnerInterceptor implements EnhanceInnerInterceptor {
    @Override public EnhancePhase phase() { return EnhancePhase.OBSERVATION; }

    private final List<SqlObservationSink> sinks = new CopyOnWriteArrayList<>();

    public SqlObservationInnerInterceptor() {
        ServiceLoader.load(SqlObservationSink.class).forEach(this::addSink);     // ★ SPI 自动发现
    }

    @Override
    public void afterExecution(Executor executor, MappedStatement ms, Object parameter, BoundSql boundSql,
                               Object result, Throwable failure, long elapsedNanos) {
        SqlObservation observation = new SqlObservation(
                ms.getId(), Objects.isNull(boundSql) ? null : boundSql.getSql(), elapsedNanos, failure);
        for (SqlObservationSink sink : sinks) {
            try {
                sink.accept(observation);
            } catch (RuntimeException exception) {
                log.warn("SQL observation sink failed: {}", sink.getClass().getName(), exception);
            }
        }
    }
}
```

### 12.2 数据类 `SqlObservation` + SPI `SqlObservationSink`

```java
public final class SqlObservation {
    private final String statementId;
    private final String sql;
    private final long elapsedNanos;
    private final Throwable failure;
    // getters + isSlow(long threshold) + toLogLine(...)
}

public interface SqlObservationSink {
    void accept(SqlObservation observation);
}
```

### 12.3 内置 sink：`SlowSqlLoggingSink`（默认）

```java
public class SlowSqlLoggingSink implements SqlObservationSink {
    private final long thresholdNanos;
    ...
    @Override
    public void accept(SqlObservation observation) {
        long elapsedNanos = observation.getElapsedNanos();
        if (elapsedNanos >= thresholdNanos) {
            log.warn("Slow SQL (>= {}ms):\n  id={}\n  sql={}\n  failure={}",
                TimeUnit.NANOSECONDS.toMillis(thresholdNanos),
                observation.getStatementId(),
                observation.getSql(),
                observation.getFailure());
        }
    }
}
```

### 12.4 扩展点：自定义 sink

实现 `SqlObservationSink`，放到 `META-INF/services/com.baomidou.mybatisplus.enhance.observation.SqlObservationSink` —— 启动时 `ServiceLoader` 自动发现。或者直接 `new SqlObservationInnerInterceptor(mySink)` 注入。

典型用户实现：把 sink 桥到 opentelemetry / prometheus / 自家 trace。

---

## 13. SQL 注入器：`EnhanceSqlInjector` + 6 个新方法

`injector/EnhanceSqlInjector.java`，48 行。继承 `DefaultSqlInjector`，**追加 6 个新方法**：

```java
@Override
public List<AbstractMethod> getMethodList(Configuration configuration, Class<?> mapperClass, TableInfo tableInfo) {
    List<AbstractMethod> methodList = super.getMethodList(configuration, mapperClass, tableInfo);
    methodList.add(new SelectIgnoreDecryptMaps());
    methodList.add(new SelectIgnoreDecryptObjs());
    methodList.add(new SelectIgnoreDecryptList());
    if (tableInfo.havePK()) {
        methodList.add(new SelectIgnoreDecryptById());
        methodList.add(new SelectIgnoreDecryptBatchIds());
        if (TableFieldHelper.getTableSignatureStoreFieldInfo(tableInfo).isPresent()) {
            methodList.add(new UpdateSignatureById());
        }
    }
    return methodList;
}
```

| 新方法 | 性质 | 适配阶段 |
|---|---|---|
| `SelectIgnoreDecryptById` | 按 ID 查询，跳过解密 | `RESULT_DECRYPTION` 前自动 bypass |
| `SelectIgnoreDecryptBatchIds` | 批查询，跳过解密 | 同上 |
| `SelectIgnoreDecryptList` | Wrapper 查询，跳过解密 | 同上 |
| `SelectIgnoreDecryptMaps` | Map 查询，跳过解密 | 同上 |
| `SelectIgnoreDecryptObjs` | 单列查询，跳过解密 | 同上 |
| `UpdateSignatureById` | 仅更新签名列（补签用） | 需 `@TableSignature(...)` |

——这些方法在 mapper 上对应 `@IgnoreEncrypted` 注解的语义：拦截器会扫描到该注解，跳过加密/解密。

---

## 14. Mapper：`EnhanceBaseMapper` + `IEnhanceService` + `EnhanceServiceImpl`

### 14.1 `EnhanceBaseMapper<T>`（95 行）

```java
public interface EnhanceBaseMapper<T> extends BaseMapper<T> {

    @IgnoreEncrypted
    T selectIgnoreDecryptById(Serializable id);

    @IgnoreEncrypted
    List<T> selectIgnoreDecryptBatchIds(Collection<? extends Serializable> idList);

    @IgnoreEncrypted
    List<Map<String, Object>> selectIgnoreDecryptMaps(Wrapper<T> queryWrapper);

    default List<T> selectIgnoreDecryptByMap(Map<String, Object> columnMap) {
        return this.selectIgnoreDecryptList(Wrappers.<T>query().allEq(columnMap));
    }

    @IgnoreEncrypted
    List<T> selectIgnoreDecryptList(Wrapper<T> queryWrapper);

    @IgnoreEncrypted
    <E> List<E> selectIgnoreDecryptObjs(Wrapper<T> queryWrapper);

    @IgnoreEncrypted
    int updateSignatureById(@Param(Constants.ENTITY) T entity);
}
```

——**7 个新方法**，第 6 行为什么缺？被 `selectIgnoreDecryptByMap` 的 default 实现代替。所有方法都带 `@IgnoreEncrypted` 注解 → 由加密/签名拦截器识别并跳过相关阶段。

### 14.2 `IEnhanceService<T>` + `EnhanceServiceImpl<M, T>`

`mybatis-plus-enhance-spring` 提供 `<T> List<T> listAll()` 等默认方法 + `@Transactional` 注解，**继承自官方 `IService` 模式**——不再额外加复杂度，只是镜像结构。

---

## 15. 上下文对象

### 15.1 `TenantContext`（已介绍）

Thread-local 当前租户 ID；set/clear 对由调用方（Sping 拦截器 / 网关）负责。

### 15.2 `SignatureUpdateContext`

```java
public class SignatureUpdateContext {
    private static final ThreadLocal<SignatureUpdateStrategy> CURRENT = new ThreadLocal<>();
    public static void setStrategy(SignatureUpdateStrategy s) { CURRENT.set(s); }
    public static SignatureUpdateStrategy current() { return CURRENT.get(); }
    public static void clear() { CURRENT.remove(); }
}
```

——配合 4 种策略使用，控制单次 update 是否重签、只更新签名、拒绝部分更新。

### 15.3 `SignatureVerificationContext`

```java
public class SignatureVerificationContext {
    private static final ThreadLocal<Boolean> IGNORED = new ThreadLocal<>();
    public static boolean isIgnored() { return Boolean.TRUE.equals(IGNORED.get()); }
    public static void ignore() { IGNORED.set(Boolean.TRUE); }
    public static void clear() { IGNORED.remove(); }
}
```

——select count(*) 这类"不想验签"的 SQL 走这条通路：在拦截器/Service 中显式 `SignatureVerificationContext.ignore()` + `try { ... } finally { SignatureVerificationContext.clear(); }`。

### 15.4 `InsertIgnoreContext` / `I18nContext`

`InsertIgnoreContext` 镜像模式：让某些 INSERT 走 MySQL `INSERT IGNORE`（需拦截器配合支持）。`I18nContext` 镜像 ThreadLocal locale。

---

## 16. 数据流 ASCII 流程图

```
                 ┌─── user code ─────────────────────┐
                 │  @Autowired EnhanceService svc;    │
                 │  svc.listAll(SecurityContext.tid)  │
                 └─────────────────┬─────────────────┘
                                   │
             ┌─────────────────────▼─────────────────────┐
             │  TenantContext.setCurrentTenantId(tenantId) │
             └─────────────────────┬─────────────────────┘
                                   │
                                   ▼
             ┌─── MBP SqlSession / Executor ──────────┐
             │  MybatisPlusEnhanceInterceptor.intercept │
             └─────────────────────┬─────────────────┘
                                   ▼
             ┌─── 阶段 100: SQL_REWRITE ─────────────┐
             │  LongSqlInnerInterceptor.beforeQuery │
             │  InsertIgnoreInnerInterceptor.beforeQuery │
             │  ┌─ TenantLineInnerInterceptor ─┐   │   ← 官方多租户
             │  │  (DefaultTenantLineHandler)   │   │
             │  └──────────────────────────────┘   │
             └─────────────────────┬─────────────────┘
                                   ▼
             ┌─── 阶段 200: PARAMETER_ENCRYPTION ──┐
             │  DataEncryptionInnerInterceptor      │
             │    .beforeQuery  ─ 实体字段加密      │
             │    .beforeUpdate ─ entity+wrapper 加密 │
             └─────────────────────┬─────────────────┘
                                   ▼
             ┌─── 阶段 300: DATA_SIGNATURE ────────┐
             │  DataSignatureInnerInterceptor      │
             │    .beforeUpdate ─ 计算签名         │
             │    .afterQuery  ─ 验签              │
             └─────────────────────┬─────────────────┘
                                   ▼
             ┌─────── executor.query / update.realExecute ────────┐
             │   (CachingExecutor → BaseExecutor → StatementHandler) │
             └─────────────────────┬───────────────────────────────┘
                                   ▼
             ┌─── 后置 4: RESULT_DECRYPTION ────────┐
             │  DataDecryptionInnerInterceptor      │
             │    .afterQuery ─ 解密已映射 POJO     │
             └─────────────────────┬─────────────────┘
                                   ▼
             ┌─── 后置 5: RESULT_I18N ──────────────┐
             │  DataI18nInnerInterceptor.afterQuery │
             │    ─ 扫描国际化字段并替换           │
             └─────────────────────┬─────────────────┘
                                   ▼
                       List<T> ready to caller
                                   │
                                   ▼
             ┌─── 后置 9: OBSERVATION ──────────────┐
             │  SqlObservationInnerInterceptor      │
             │    .afterExecution(elapsedNanos)      │
             │  → SqlObservationSink.accept          │
             │    → SlowSqlLoggingSink               │
             │    → 其他自定义 sink                   │
             └──────────────────────────────────────┘
```

---

## 17. 与 MyBatis / MyBatis-Plus / rbatis 对照速记表

| 维度 | MyBatis-Plus Enhance | MyBatis-Plus（baomidou） | MyBatis 3 | rbatis / rbatis-plus |
|---|---|---|---|---|
| 定位 | 第三方企业级增强层 | 官方扩展框架 | 主流程 | Rust 移植 |
| 拦截器命名 | `EnhanceInnerInterceptor`（继承自 `InnerInterceptor`） | `InnerInterceptor` | `Interceptor` | `Intercept` trait |
| 阶段顺序 | **`EnhancePhase` order 100→900** 编译期校验 | 顺序约定（无校验） | 顺序约定 | trait 链 push 顺序 |
| 加解密 | 内置 **AES/SM4 + HMAC** 字段级 + `@IgnoreEncrypted` | 无 | 无 | 不涉及 |
| 表签名 | `@TableSignature` + HMAC | 无 | 无 | 不涉及 |
| 多租户 | 沿用官方 `TenantLineHandler`，提供默认适配 | 官方内置 | 无 | trait ctx 元数据 |
| 数据权限 | `@DataScopePlus` + `DataScopeExpressionProvider` SPI | 第三方独立包 | 无 | 不涉及 |
| 国际化 | 完整 bundle 抽象 + LocaleProvider | 无 | 无 | 不涉及 |
| SQL 观测 | `SqlObservationSink` + `ServiceLoader` 自发现 | 部分内置 | 无 | 不涉及 |
| Mapper 增强 | `@IgnoreEncrypted` 注解 + 7 个新方法 | 注解 + 30+ 默认方法 | 用户写 SQL | `crud!{}` 宏 |
| 事务 | Spring `@Transactional` | 同 | 同 | trait 事件总线 |
| 注解风格 | `@IgnoreEncrypted`/`@TableSignature`/`@DataScopePlus` | `@TableName`/`@Version`/`@Interceptors` | MyBatis 自带 | trait + 宏 |

---

## 18. 关键设计权衡（FAQ）

### Q1：为什么要"二次分发"+ 阶段 order 数？

每个拦截器是独立 InnerInterceptor，**`@Intercepts` 都在 `Executor.query` 上**。如果官方 `MybatisPlusInterceptor` 直接 `invocation.proceed()`，就拿不到 **`List<Object> result`**。所以增强版必须分两段：
- **前段**：复用官方 `beforeQuery / beforeUpdate / beforePrepare / beforeGetBoundSql`，但这次直接自己掉 `executor.query(...)`/`invocation.proceed()`
- **后段**：调用 `afterQuery / afterUpdate / afterExecution` 三个增强钩子

### Q2：`phase()` 顺序校验为什么抛 `IllegalArgumentException` 而不是 warn？

顺序写错 → 框架的目标语义根本不能实现（如：参数还没加密就去签名 → 签的是明文），属于**配置错误不是运行时错误**。fail-fast 最合适。

### Q3：为什么 SQL 观测用 `ServiceLoader` 自动发现？

业务方实现 sink 后不必改业务代码，只需要在 `META-INF/services/...SqlObservationSink` 声明实现。**热插拔 / 多 sink 收集**都不需要 modify existing code。这是 JDK SPI 的经典用法。

### Q4：为什么签名拦截器的 entity 类型解析要 fallback 到 `GenericTypeUtils.resolveTypeArguments(...)`？

MyBatis `parameterObject` 可能是 `Map` / `Wrapper` / `Entity` / 简单类型；框架**没有统一的入口**调用，告诉它"这是哪个表的 UPDATE"。所以用 3 层 fallback：
1. `paramMap.get(ENTITY)` → 实体
2. `wrapper.getEntityClass()` → 实体
3. statementId → mapper class → `BaseMapper<T>` 的 T

——`OptimisticLockerInnerInterceptor` 也是这个回退模式（MyBatis-Plus 自身文档就已说明）。

### Q5：表签名 `signSwitch + signVerify` 独立开关的设计价值是什么？

历史数据迁移 4 阶段：
1. **关关** = 无签名（默认）
2. **开关** = 只写新签名，历史未签
3. **开开** = 写 + 验，但历史数据验签失败
4. **离线重签 → 关开** = 全表已签才开验

——开关独立让"渐进式迁移"**无需切换版本**完成。

### Q6：`@IgnoreEncrypted` 注解放在 Mapper 方法上是不是与"字段标记"矛盾？

`@IgnoreEncrypted` 是 **方法级**：某些业务（比如后台运维、按哈希搜索）需要原始密文；其他业务都默认解密。框架在 beforeQuery + beforeUpdate **反射读 mapper.getMethod(...)**：

```java
String mappedStatementId = ms.getId();
Class<?> mapperClazz = Class.forName(mappedStatementId.substring(0, mappedStatementId.lastIndexOf(".")));
String methodName = mappedStatementId.substring(mappedStatementId.lastIndexOf(".") + 1);
Method method = ReflectUtil.getMethodByName(mapperClazz, methodName);
IgnoreEncrypted ignoreEncrypted = AnnotationUtils.findFirstAnnotation(IgnoreEncrypted.class, method);
```

—— **注解位置** ≠ 字段级加密开关（字段级由反射读 `@EncryptField`）。两者并列存在。

### Q7：`EnhanceServiceImpl` 在 spring 模块只有 2 个 java 文件，它做了什么？

Spring 模块只做"加 `@Transactional` + 继承 `ServiceImpl`"。这意味着 **90% 的扩展发生在 Extension 模块**——轻耦合，最小化对 Spring 环境的依赖。

### Q8：自定义 `phase = UNSPECIFIED` 时与官方 `InnerInterceptor` 兼容吗？

兼容。框架强制顺序校验时**跳过** UNSPECIFIED。但用户依然可以把任意自定义 SPI 放在链上某个位置。

### Q9：与 rbatis-plus 的关联：rbatis-plus 何时需要"增强层"？

`rbatis-plus`（Rust）的增强层可以**结构相同**——`Intercept` trait 上加 after 钩子 + `phase::ord()` 枚举 + 顺序校验 + ServiceLoader 风格的 sink。但**实现加重**（AES / SM4 等）则要依赖 Rust crates `aes-gcm` / `sm4` / …。

---

## 19. codegraph 速查命令

```bash
export PATH="/Users/wandl/.nvm/versions/node/v24.18.0/bin:$PATH"
cd /Users/wandl/workspaces/workspace-github-easy-4-java/mybatis-plus-enhance

codegraph status                    # 105 文件
codegraph query "EnhancePhase"
codegraph query "EnhanceInnerInterceptor\|MybatisPlusEnhanceInterceptor"
codegraph query "DataEncryptionInnerInterceptor\|DataDecryptionInnerInterceptor"
codegraph query "DataSignatureInnerInterceptor\|DefaultDataSignatureHandler"
codegraph query "IgnoreEncrypted\|EncryptedFieldHandler\|CryptoKeyProvider"
codegraph query "TenantContext\|TenantLineHandler"
codegraph query "DataScopePlus\|DataScopeAnnotationHandler\|DataScopeExpressionProvider"
codegraph query "DataI18nInnerInterceptor\|MultipleResourceBundle"
codegraph query "LongSqlInnerInterceptor\|InsertIgnoreInnerInterceptor"
codegraph query "SqlObservation\|SqlObservationSink\|SlowSqlLoggingSink"
codegraph query "EnhanceBaseMapper\|EnhanceSqlInjector\|IEnhanceService\|EnhanceServiceImpl"
codegraph query "SignatureUpdateContext\|SignatureVerificationContext\|InsertIgnoreContext"
```

---

## 20. 推荐阅读顺序

1. **`mybatis-plus-enhance-core/plugins/inner/EnhancePhase.java`**（46 行）—— 阶段枚举的全文
2. **`mybatis-plus-enhance-core/plugins/inner/EnhanceInnerInterceptor.java`**（91 行）—— 4 个钩子签名
3. **`mybatis-plus-enhance-core/plugins/MybatisPlusEnhanceInterceptor.java`**（263 行）—— `intercept()` + `executeQuery()` + `executeUpdate()` + `notifyAfterExecution()`
4. **`mybatis-plus-enhance-extension/plugins/inner/DataEncryptionInnerInterceptor.java`** + **`DataSignatureInnerInterceptor.java`** + **`DataDecryptionInnerInterceptor.java`** —— 三个核心 pair
5. **`mybatis-plus-enhance-extension/observation/{SqlObservation,SlowSqlLoggingSink}.java`** —— SQL 观测最小实现
6. **`mybatis-plus-enhance-extension/tenant/DefaultTenantLineHandler.java`** + **`context/TenantContext.java`** —— 最小线程上下文
7. **`mybatis-plus-enhance-extension/i18n/interceptor/DataI18nInnerInterceptor.java`** —— 国际化精简版
8. **`mybatis-plus-enhance-extension/mapper/EnhanceBaseMapper.java`** + **`injector/EnhanceSqlInjector.java`** —— SQL 注入
9. **`mybatis-plus-enhance-spring/service/IEnhanceService.java`** + **`impl/EnhanceServiceImpl.java`** —— Spring 集成

---

## 21. 已废弃 / 注意点 / 关联注解

### 21.1 注意点

1. **版本三线并存**：`1.0.x / 2.0.x / 3.0.x` 各对应 `javax.*` / `jakarta.*` / `jakarta.*` —— **不要混用 jar**，否则 `NoSuchMethodError`
2. **CI Friendly Version**：CI 中 `${revision}` 会展开为实际版本号；本地开发期任意指定
3. **加密/签名必须按顺序**：如果有签名，必须有加密 → 否则表签名覆盖的是明文没意义
4. **`@IgnoreEncrypted` 注解存在 mapper interface method 而非 mapper.xml ID** —— MyBatis 在 proxy 调用时通过 `ms.getId()` 反查 method
5. **签名 "REJECT_PARTIAL"**：在 `signSwitch = true` 的签名表 + 局部更新场景自动抛错。补签需用 `updateSignatureById(et)` 整行签
6. **slow SQL log 仅 log warn**：不限制日志框架；slf4j / logback 任一即可
7. **`MybatisPlusEnhanceInterceptor` 与官方 `MybatisPlusInterceptor` 二选一**：同时注册会重复拦截一次 Executor（同样的 5 个 `@Signature`）。若只要官方能力，请直接用 `MybatisPlusInterceptor`
8. **ServiceLoader 时序**：必须在 Spring 容器初始化之前把 sink 注册到 classpath —— 否则 `SqlObservationInnerInterceptor` 自动发现时拿不到

### 21.2 关联依赖（"mybatis-enhance-annotation" 包）

README 写道：

> "公共加密和国际化注解统一复用 `mybatis-enhance-annotation`，MyBatis-Plus 项目不维护重复协议；唯一的 Plus 专属数据权限注解随 Extension 提供，不为单个注解拆分独立 Maven 模块。"

所以 `@IgnoreEncrypted` 等可能来自外部 `io.github.hiwepy:mybatis-enhance-annotation` Maven 工件；本仓库只**使用**不**拥有**这类注解。

### 21.3 未来（来自 README / GitHub issues）

- 兼容 `MyBatis-Plus 3.6.x` 新版 LineHandler SPI
- 接入 opentelemetry / micrometer 的内置 sink
- 与国产数据库方言（人大金仓 / 达梦）的字段加解密兼容

---

## 22. 与 **rbatis-plus** 移植 checklist

把这份 Java 设计平移到 Rust（`/Users/wandl/workspaces/workspace-github-easy-4-rust/rbatis-plus`）时，对照表：

| Java 资产 | Rust 可参考 API |
|---|---|
| `EnhancePhase` enum | `pub enum InterceptPhase { SqlRewrite=100, Encryption=200, Signature=300, Decryption=400, I18n=500, Observation=900 }` |
| `EnhanceInnerInterceptor` trait (4 钩子) | `pub trait EnhanceIntercept: Intercept { async fn phase(&self); async fn after_query(...); async fn after_update(...); async fn after_execution(...); }` |
| `MybatisPlusEnhanceInterceptor` 调度器 | `pub struct MybatisPlusEnhanceInterceptor { interceptors: Vec<Arc<dyn EnhanceIntercept>> }` 含 `register(&mut self, ...)` 时序校验 |
| `DataEncryptionInnerInterceptor` | 用 `aes-gcm` / `sm4` crate + 字段反射元数据 |
| `SignatureUpdateStrategy` | 编译期 enum 即可 |
| `ServiceLoader<SqlObservationSink>` | Rust 的 trait object + lazy_static 注册表 |
| `EnhanceBaseMapper<T>` | `pub trait EnhanceBaseMapper<T>: BaseMapper<T> { @IgnoreEncrypted fn select_ignore_decrypt_by_id(...) ... }` + 宏 |
| `DefaultTenantLineHandler + TenantContext` | `thread_local! { static CURRENT_TENANT: RefCell<Option<...>> = ... }` |
| `@IgnoreEncrypted` 注解 | Rust 的 `#[ignore_encrypted] async fn ...` 过程宏 |
| `SignatureVerificationContext.ignore()` | `scoped_thread_local!` |

——这个对照可直接当 `rbatis-plus` 后续工作的 checklist。

---

## 附录 A：异常路径与降级

`MybatisPlusEnhanceInterceptor.executeQuery` 内：

```java
try {
    result = executor.query(...);
    ...
} catch (Throwable throwable) {
    failure = throwable;
    throw throwable;
} finally {
    if (elapsedNanos == 0L) elapsedNanos = System.nanoTime() - startedAt;
    notifyAfterExecution(executor, ms, parameter, boundSql, result, failure, elapsedNanos);
}
```

——异常**透传**，但 `afterExecution` 总会跑 → **保证慢 SQL/失败 SQL 一律被观测到**。这是企业级观测的关键设计。

---

## 附录 B：8 个拦截器的"本质"对照

| 拦截器 | "前"做了什么 | "后"做了什么 |
|---|---|---|
| `DataEncryption` | 字段 → 密文 | — |
| `DataSignature` | entity → 签名 | 结果验签 |
| `DataDecryption` | — | 密文 → 字段 |
| `DataI18n` | — | 字典替换 |
| `LongSql` | 检测 SQL 长度 | — |
| `InsertIgnore` | 改写 SQL 为 INSERT IGNORE | — |
| `SqlObservation` | — | 时长 + SqlObservation 广播 |
| (官方) `TenantLine` | SQL 拼接 tenant_id = ? | — |

——每一次拦截都"不越界"：pre/after 钩子的设计就是为了**让增强逻辑按阶段严格分开**，避免出现"加密后又验签"或"签名后解密"导致数据被破坏。