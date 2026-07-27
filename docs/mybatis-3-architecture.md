# MyBatis 3 架构与代码导读

> 本文档基于本地 **codegraph** 索引（`/Users/wandl/workspaces/workspace-github/mybatis-3`，1396 个 Java 文件、约 14k 个符号节点）的一手源码梳理。
>
> 参考资料：
> - GitHub：<https://github.com/mybatis/mybatis-3>
> - 本地路径：`/Users/wandl/workspaces/workspace-github/mybatis-3`
> - 本地版本：`mybatis 3.6.0-SNAPSHOT`（继承自 `mybatis-parent 52`）
>
> ---
>
> ## 在这里的原因
>
> 本文档路径：`mybatis-3-architecture.md`。
>
> 因为它是 **rbatis-plus Rust 移植** 的设计基石之一：
> - **MyBatis 3** = `rbatis` 主仓库的设计原型（`Executor` / `SqlSession` / `MapperProxy` / `Plugin` / `Cache` SPI）
> - **MyBatis-Plus**（同父仓库但 Java 版本）= Service / CRUD / Wrapper / InnerInterceptor 模板
>
> 只有 **同时了解这两份 Java 设计**，才能把握 `rbatis-plus` 应该长什么样：
>
> - 想把 `BaseMapper<T>` 改成 Rust trait + 宏？看本仓库的 **MyBatis-Plus** 文档（`../mybatis-plus-architecture.md`）即可找到"接口继承 + 泛型 T"路径
> - 想把 Java 拦截器 `Plugin.wrap` + `Executor` 拦截链转成 Rust 过程宏？读 **本文**（`Executor` + `Plugin.wrap` + JDBC 路径）找到 Java 真实的拦截点
> - 想把 `InnerInterceptor` 生命周期映射到 Rust？看 **MyBatis-Plus** 文档 §7 与 **本文** §4 的拦截链对照
>
> 两份文档互相引用见末尾"相关阅读"。

---

## 目录

1. 一句话定位
2. 仓库布局与规模
3. 核心抽象：`SqlSession` + `Configuration` + `MappedStatement`
4. 执行器体系（Executor）
   - 4.1 `Executor` 接口（17 个方法）
   - 4.2 `BaseExecutor` 抽象基类（L1 + DeferredLoad + 嵌套查询栈）
   - 4.3 `SimpleExecutor` / `ReuseExecutor` / `BatchExecutor` 三个具体实现
   - 4.4 `CachingExecutor` 二级缓存装饰器
5. 二级缓存（Cache）
   - 5.1 `Cache` 接口（7 个方法）
   - 5.2 `CacheKey`（基于 list + 异或乘法哈希）
   - 5.3 装饰器栈：10 个 `decorators/*`
   - 5.4 `TransactionalCache` + `TransactionalCacheManager`（MyBatis 二级事务核心）
6. 拦截器（Plugin）
7. Mapper 绑定（Binding）
8. 结果处理：参数解析、反射、TypeHandler、ResultSet、Statement
9. 数据源：JNDI / Unpooled / Pooled
10. 数据流全链路 ASCII 流程图
11. 与 rbatis / rbatis-cache 对照速记表
12. 关键设计权衡（FAQ）
13. codegraph 速查命令
14. 推荐阅读顺序
15. 未变 / 已废弃 / 注意点

---

## 1. 一句话定位

**MyBatis = 半自动 SQL-ORM + 反射驱动的 "SQL mapper"。**

> SQL 仍写在 XML 或注解里，**MyBatis 负责把它们映射到 Java 方法调用 + 结果集到 POJO**。它不试图帮用户写 SQL、提供完整的实体生命周期管理，与 Hibernate 等 "full ORM" 形成对照。

设计哲学可以归纳成 4 条：

1. **SQL 始终是 SQL**：MyBatis 不阻止用户写任何 SQL（甚至 `CallableStatement` 也支持）
2. **接口即映射**：用 Java 接口 + 注解 / XML 绑定；运行时用 JDK 动态代理生成实现
3. **分层抽象**：从 `SqlSession` → `Executor` → `StatementHandler` → JDBC `Statement`
4. **可插拔**：插件（Interceptor + Plugin Proxy）、缓存（Cache SPI + Decorator）、TypeHandler、DataSource 全部可替换

---

## 2. 仓库布局与规模

```
mybatis-3/                                                Maven root (parent = mybatis-parent:52)
├── pom.xml                                          ── mybatis 3.6.0-SNAPSHOT
├── src/main/java/org/apache/ibatis/                 ── 393 主源码 Java 文件
│   ├── annotations/               (~ 32 个注解)       @Select / @Insert / @Update / @Delete / @Param / ...
│   ├── binding/                                    ── 7 个 Mapper 绑定类  ~ 770 行
│   │   ├── MapperRegistry.java                  (124) 注册表 + getMapper 工厂
│   │   ├── MapperProxy.java                     (121) JDK 动态代理
│   │   ├── MapperProxyFactory.java              ( 55) 缓存每个 Mapper 的代理
│   │   ├── MapperMethod.java                    (386) 单方法签名 + 反射调用 SqlSession
│   │   └── MapperMethodInvoker.java             ( 26) 接口化方法 invocation
│   ├── builder/                                     ── 13 类 ~ 1000 行（XML / 注解映射构造）
│   │   ├── BaseBuilder.java                     公共父类
│   │   ├── SqlSourceBuilder.java                把 #{var} 解析成 ? 占位
│   │   ├── annotation/MapperAnnotationBuilder    (  ~ 290) 注解式 mapper 构造
│   │   └── xml/{XMLConfigBuilder, XMLMapperBuilder, XMLStatementBuilder, ...}
│   ├── cursor/                                       流式结果游标
│   ├── datasource/                                ── JNDI / Unpooled / Pooled
│   ├── exceptions/                                ── 7 种异常
│   ├── executor/                                  ── 核心
│   │   ├── Executor.java                       17 个方法
│   │   ├── BaseExecutor.java                   L1 + DeferredLoad + 嵌套查询栈
│   │   ├── CachingExecutor.java                二级缓存装饰
│   │   ├── SimpleExecutor.java / ReuseExecutor.java / BatchExecutor.java 三个具体
│   │   ├── parameter/    (4 类)
│   │   ├── result/       (5 类)
│   │   ├── resultset/    (8 类)
│   │   ├── statement/    (StatementHandler + 4 个 StatementHandler 实现)
│   │   ├── keygen/       (Jdbc3KeyGenerator / SelectKeyGenerator)
│   │   └── loader/       (ResultLoader + Javassist/Cglib 延迟加载代理工厂)
│   ├── logging/                                   ── 7 类日志 facade + JDBC connection logger
│   ├── mapping/                                   ── BoundSql / MappedStatement / ParameterMapping / SqlSource / ResultMap / ...
│   ├── parsing/                                   ── XPath + 通用 Property 表达式
│   ├── plugin/                                    ── 8 类（Interceptor / Plugin Proxy / Invocation / Signature）
│   ├── reflection/                                ── 反射工具 (MetaObject / Reflector / ParamNameResolver / TypeParameterResolver / ...)
│   ├── scripting/                                 ── OGNL / Velocity / Kotlin SqlSource provider
│   ├── session/                                   ── SqlSession / SqlSessionFactory / Configuration / ResultContext / RowBounds
│   ├── transaction/                               ── JdbcTransaction / ManagedTransactionFactory
│   ├── type/                                      ── TypeHandler 体系 + 30+ 内置 handler
│   └── cache/                                     ── 5 类接口 + 11 个装饰器 + PerpetualCache
└── src/test/java/org/apache/ibatis/                ── 1002 个测试文件
```

codegraph 总览：

```
Files:     1396  (Java 1396, xml 441, yaml 9, properties 7)
Nodes:     大约 14,000+（含 function/interface/class 计数显示仅 function 8，因此大量为 class/interface）
Edges:     巨大（SQL-Influenced 边可期）
```

> 注：codegraph 默认对 Java 解析较深，但本目录体量太大（`src` 11M）——大量细节以文本方式给出。

---

## 3. 核心抽象：`SqlSession` + `Configuration` + `MappedStatement`

### 3.1 `SqlSession`（`session/SqlSession.java`，17 个业务方法）

```java
public interface SqlSession extends Closeable {
  <T> T selectOne(String statement);                         // 重载 1
  <T> T selectOne(String statement, Object parameter);
  <E> List<E> selectList(String statement, Object parameter, RowBounds rowBounds);
  <K, V> Map<K, V> selectMap(String statement, Object parameter, String mapKey);
  <T> Cursor<T> selectCursor(String statement, Object parameter, RowBounds rowBounds);
  void select(String statement, Object parameter, RowBounds rowBounds, ResultHandler handler);
  int insert/update/delete(String statement, Object parameter);
  void commit(boolean force);
  void rollback(boolean force);
  List<BatchResult> flushStatements();
  void close();
  void clearCache();                                         // 清 L1+L2
  Configuration getConfiguration();
  <T> T getMapper(Class<T> type);                            // ★ 通过 binding 包走代理
  Connection getConnection();
}
```

**默认实现**：`session/defaults/DefaultSqlSession.java`，**所有方法直接转发给内部的 `Executor`**，只做"SQL ID → MappedStatement 查找 → 参数转换 → 调用 Executor + 异常翻译（PersistenceException）"。

### 3.2 `Configuration`（`session/Configuration.java`，1200 行，**整个 MyBatis 的中心**）

`Configuration` 是根容器（**唯一静态可变**），保存：

| 子组件 | 作用 |
|---|---|
| `environment` + `DataSourceFactory` | 数据源配置 |
| `transactionFactory` | 事务工厂 |
| `interceptorChain` | 全局拦截器链 |
| `mappedStatements` | `Map<String, MappedStatement>` SQL ID → 执行计划 |
| `resultMaps` | `Map<String, ResultMap>` |
| `parameterMaps` | `Map<String, ParameterMap>` |
| `typeHandlerRegistry` | 全局 TypeHandler 注册表 |
| `typeAliasRegistry` | 别名 (e.g. `user` → `com.example.User`) |
| `objectFactory` / `objectWrapperFactory` / `reflectorFactory` | 反射相关 |
| `caches` | 全局 `Map<String, Cache>` L2 cache 实例 |
| `cacheRefs` | `CacheRef` 全局引用 |
| `incompleteStatements` / `incompleteCacheRefs` | 解析中间态 |
| `loadedResources` | 已加载的 XML mapper 路径 |
| `localCacheScope` | `STATEMENT` 或 `SESSION` 控制 L1 何时清理 |
| `defaultFetchSize` / `defaultStatementTimeout` / `safeRowBoundsEnabled` 等 | 全局默认值 |

### 3.3 `MappedStatement`（`mapping/MappedStatement.java`）

一条 SQL 的"编译产物"，最终被 `StatementHandler` 使用：

```java
public final class MappedStatement {
  private Configuration configuration;
  private String id;                  // 全局唯一（"namespace.statementId"）
  private SqlCommandType sqlCommandType;   // SELECT/INSERT/UPDATE/DELETE/FLUSH
  private SqlSource sqlSource;             // 动态 SQL
  private Cache cache;                     // 关联 L2 实例（来自 @CacheNamespace 或 <cache>）
  private boolean useCache;
  private boolean flushCacheRequired;
  private boolean resultOrdered;
  private StatementType statementType;     // STATEMENT / PREPARED / CALLABLE
  private int fetchSize;
  private int timeout;
  private RowBounds rowBounds;
  private ParameterMap parameterMap;       // MyBatis 中正在淡化
  // …略
}
```

### 3.4 `BoundSql`（`mapping/BoundSql.java`）

绑定后的 SQL，包含：实际 SQL 字符串 + 参数映射列表 + 参数对象。

---

## 4. 执行器体系（Executor）

### 4.1 `Executor` 接口（`executor/Executor.java:33-69`）

17 个方法，关键 8 个：

```java
public interface Executor {
  ResultHandler NO_RESULT_HANDLER = null;
  int update(MappedStatement ms, Object parameter) throws SQLException;
  <E> List<E> query(MappedStatement ms, Object parameter, RowBounds rowBounds,
                    ResultHandler resultHandler, CacheKey cacheKey, BoundSql boundSql) throws SQLException;
  <E> List<E> query(MappedStatement ms, Object parameter, RowBounds rowBounds, ResultHandler resultHandler) throws SQLException;
  <E> Cursor<E> queryCursor(MappedStatement ms, Object parameter, RowBounds rowBounds) throws SQLException;
  List<BatchResult> flushStatements() throws SQLException;
  void commit(boolean required) throws SQLException;
  void rollback(boolean required) throws SQLException;

  CacheKey createCacheKey(MappedStatement ms, Object parameterObject, RowBounds rowBounds, BoundSql boundSql);
  boolean isCached(MappedStatement ms, CacheKey key);

  void clearLocalCache();
  void deferLoad(MappedStatement ms, MetaObject resultObject, String property, CacheKey key, Class<?> targetType);

  Transaction getTransaction();
  void close(boolean forceRollback);
  boolean isClosed();
  void setExecutorWrapper(Executor executor);
}
```

注意：`setExecutorWrapper` 只用于 `CachingExecutor` 内部装饰其它 Executor 的反向引用——它自己抛 `UnsupportedOperationException`。

### 4.2 `BaseExecutor`（抽象基类，约 700 行）

**所有 Executor 中"骨架 + L1 缓存 + 嵌套栈"都在它**。

```java
public abstract class BaseExecutor implements Executor {

  protected Transaction transaction;
  protected Executor wrapper;                          // CachingExecutor 装饰回来用

  protected ConcurrentLinkedQueue<DeferredLoad> deferredLoads;
  protected PerpetualCache localCache;                 // ★ L1
  protected PerpetualCache localOutputParameterCache;  // 存储过程的 out 参数缓存
  protected Configuration configuration;

  protected int queryStack;                            // ★ 嵌套查询栈深度
  private boolean closed;
  ...
}
```

#### 4.2.1 L1：PerpetualCache

`localCache` 是裸的 `HashMap`（`PerpetualCache` 是其实现），无容量限制、**由 `Configuration.localCacheScope` 决定何时清理**：

- `SESSION`（默认）：一次 session 内不自动清，全靠 commit/rollback/update 时显式清
- `STATEMENT`：`query()` 退出栈（`queryStack==0`）时立即清（issue #482）

`query()` 关键流程（`BaseExecutor.java:142-176`）：

```java
if (queryStack == 0 && ms.isFlushCacheRequired()) {
    clearLocalCache();                                  // issue #482 入口
}
List<E> list;
try {
    queryStack++;
    list = resultHandler == null ? (List<E>) localCache.getObject(key) : null;
    if (list != null) {
        handleLocallyCachedOutputParameters(ms, key, parameter, boundSql);
    } else {
        list = queryFromDatabase(ms, parameter, rowBounds, resultHandler, key, boundSql);
    }
} finally {
    queryStack--;
}
if (queryStack == 0) {
    for (DeferredLoad deferredLoad : deferredLoads) {
        deferredLoad.load();                            // 嵌套完成 → 触发延迟加载
    }
    deferredLoads.clear();
    if (configuration.getLocalCacheScope() == LocalCacheScope.STATEMENT) {
        clearLocalCache();                              // issue #482 出口
    }
}
```

#### 4.2.2 DeferredLoad（延迟加载）

每个嵌套结果可以标记"延迟加载"——`deferLoad(...)` 把任务塞入 `deferredLoads`，**直到外层 query 出栈（栈底）才批量加载**，避免一边查询一边深加载把 resultset 关掉。

#### 4.2.3 `queryFromDatabase`

`BaseExecutor` 没有具体 `doQuery()`，留给 3 个子类（Simple/Reuse/Batch）实现。`queryFromDatabase` 走完整流程：

1. `localCache.putObject(EXECUTION_PLACEHOLDER)` 先占位（防同 session 嵌套同一 key 死递归）
2. 调用子类 `doQuery()`
3. `localCache.putObject(key, list)` 写入
4. 调用 `DefaultResultHandler` 处理结果

### 4.3 三个具体实现

| 实现 | 文件 | 行为 |
|---|---|---|
| `SimpleExecutor` | `executor/SimpleExecutor.java` | 默认。每次 `doQuery()` 创建一个新的 `Statement`，用完即关 |
| `ReuseExecutor` | `executor/ReuseExecutor.java` | 内部 `Map<String, Statement>` 缓存，按 SQL 文本复用 `Statement`（**仅当 SQL 完全相同**） |
| `BatchExecutor` | `executor/BatchExecutor.java` | 累积 `update`，一次性 `executeBatch()`；需要显式调用 `flushStatements()` |

选哪个由 `Configuration.executorType` 决定（`session/ExecutorType.java: SIMPLE / REUSE / BATCH`）。

### 4.4 `CachingExecutor`（`executor/CachingExecutor.java:39-180`）

**装饰器模式**——包到具体 `BaseExecutor` 外层，提供 L2：

```java
public class CachingExecutor implements Executor {
  private final Executor delegate;
  private final TransactionalCacheManager tcm = new TransactionalCacheManager();
  ...
}
```

```java
<E> List<E> query(MappedStatement ms, ..., CacheKey key, BoundSql boundSql) throws SQLException {
    Cache cache = ms.getCache();
    if (cache != null) {
        flushCacheIfRequired(ms);
        if (ms.isUseCache() && resultHandler == null) {
            ensureNoOutParams(ms, boundSql);                    // 拒绝 CALLABLE 的 OUT
            List<E> list = (List<E>) tcm.getObject(cache, key);
            if (list == null) {
                list = delegate.query(ms, parameterObject, rowBounds, resultHandler, key, boundSql);
                tcm.putObject(cache, key, list);                // issue #578 / #116
            }
            return list;
        }
    }
    return delegate.query(ms, parameterObject, rowBounds, resultHandler, key, boundSql);
}
```

`flushCacheIfRequired` 处理 update 查询：若 `ms.isFlushCacheRequired()` → `tcm.clear(cache)`。

`close/commit/rollback` 同步调用 `tcm` 的对应操作：

```java
void close(boolean forceRollback) {
    try {
        if (forceRollback) { tcm.rollback(); }
        else               { tcm.commit();   }
    } finally {
        delegate.close(forceRollback);
    }
}
```

装饰链结构：

```
Configuration.newExecutor → CachingExecutor(SimpleExecutor/ReuseExecutor/BatchExecutor)
```

---

## 5. 二级缓存（Cache）

### 5.1 `Cache` 接口（`cache/Cache.java`）

7 个方法（极简 SPI）：

```java
public interface Cache {
    String getId();
    void putObject(Object key, Object value);
    Object getObject(Object key);
    Object removeObject(Object key);                       // 仅 rollback 时被调用来释放 BlockingCache 锁
    void clear();
    int getSize();                                          // 可选
    default ReadWriteLock getReadWriteLock() { return null; }   // 已 noop（任何锁在 Cache 自身管）
}
```

**关键约束**：缓存实现必须有构造函数 `MyCache(String id)`——MyBatis 把 namespace 作为 id 传进去。这是为什么 `caffeine-cache` 仓库那个 `CaffeineCache` 只有一个 38 行 `CaffeineCache(String id)` 构造。

### 5.2 `CacheKey`（`cache/CacheKey.java`）

不是 hash digest 也不是单一字段，而是个**有序分量列表**：

```java
public class CacheKey implements Cloneable, Serializable {
    public static final CacheKey NULL_CACHE_KEY = new CacheKey() { ... 永远不更新 ... };
    private final int multiplier;          // 37
    private int hashcode;                  // 17 (初始)
    private long checksum;                 // components 各自 hash 之和
    private int count;
    private List<Object> updateList;

    public void update(Object object) {                    // 把组件加入 updateList
        int baseHashCode = object == null ? 1 : ArrayUtil.hashCode(object);
        count++;
        checksum += baseHashCode;
        baseHashCode *= count;
        hashcode = multiplier * hashcode + baseHashCode;
        updateList.add(object);
    }
    public boolean equals(Object o) {                      // 三段相等
        if (hashcode != other.hashcode) return false;
        if (checksum != other.checksum) return false;
        if (count != other.count) return false;
        for (int i = 0; i < updateList.size(); i++) {
            if (!ArrayUtil.equals(thisObject, thatObject)) return false;
        }
        return true;
    }
}
```

**设计要点**：

- 用 `List` 累加每条边界，hashcode + checksum + count + 逐元素 equals 的**四重门**校验——比 rbatis 的 digest + full-key 校验更朴素，但 Java 重对象下也能快速排除
- 不可变的 `NULL_CACHE_KEY`：代表"无条件跳过缓存"的标记，调用 `update()` 会抛异常

### 5.3 装饰器栈（11 个 `decorators/*`）

| 装饰器 | 行 | 作用 |
|---|---:|---|
| `PerpetualCache` (在 `cache/impl/`) | ~ 65 | 基础 `HashMap<Object, Object>` |
| `LruCache` | ~ 80 | LRU 容量限制 |
| `FifoCache` | ~ 60 | FIFO 容量限制 |
| `SoftCache` | ~ 100 | SoftReference，GC 收回时清理队列 |
| `WeakCache` | ~ 110 | WeakReference |
| `ScheduledCache` | ~ 60 | 定时清空 |
| `BlockingCache` | ~ 130 | **ConcurrentHashMap<key, CountDownLatch> 单飞**（带 timeout） |
| `SerializedCache` | ~ 80 | value 序列化/反序列化 |
| `SynchronizedCache` | ~ 60 | 全局 `synchronized` |
| `LoggingCache` | ~ 60 | 命中数/请求数统计 |
| `TransactionalCache` | ~ 135 | **MyBatis L2 事务核心**——下文详 |

#### 5.3.1 `BlockingCache`（`BlockingCache.java:37-127`）

非常 direct 的实现：

```java
@Override
public Object getObject(Object key) {
    acquireLock(key);
    Object value = delegate.getObject(key);
    if (value != null) { releaseLock(key); }
    return value;
}

@Override
public void putObject(Object key, Object value) {
    try {
        delegate.putObject(key, value);
    } finally {
        releaseLock(key);                       // 即使 put 失败也释放锁
    }
}
```

`acquireLock` 用 `ConcurrentHashMap.putIfAbsent(key, CountDownLatch(1))`：

- 若 `null` 表示新建 latch 占为己有
- 若有现有 latch → `latch.await(timeout)` 等待

`removeObject(Object key)` 的语义"despite its name, this method is called only to release locks"——`CachingExecutor.rollback` 的 `tcm.rollback()` → `TransactionalCache.rollback()` → `unlockMissedEntries()` → `delegate.removeObject(entry)`，调用者用它**来释放锁**。这是双栈约定。

#### 5.3.2 ⚠️ 死锁警告

源码注释里直接写："By its nature, this implementation can cause deadlock when used incorrectly"。**MyBatis 文档明确**推荐把 `BlockingCache` 放在装饰链最外层。

### 5.4 `TransactionalCache`（`cache/decorators/TransactionalCache.java`）

**这是 MyBatis L2 真正的"事务"概念**——介于 `CachingExecutor` 与底层 `Cache` 之间：

```java
public class TransactionalCache implements Cache {
    private final Cache delegate;
    private boolean clearOnCommit;
    private final Map<Object, Object> entriesToAddOnCommit;   // 待 commit flush
    private final Set<Object> entriesMissedInCache;          // block cache 释放锁用

    public Object getObject(Object key) {
        Object object = delegate.getObject(key);
        if (object == null) entriesMissedInCache.add(key);
        if (clearOnCommit) return null;                     // 问题 #146
        return object;
    }
    public void putObject(Object key, Object object) {
        entriesToAddOnCommit.put(key, object);              // 只入缓冲区，**不立即写**!
    }
    public void clear() {
        clearOnCommit = true;
        entriesToAddOnCommit.clear();
    }
    public void commit() {
        if (clearOnCommit) { delegate.clear(); }
        flushPendingEntries();                              // 先 flush 再 reset
        reset();
    }
    public void rollback() {
        unlockMissedEntries();                              // 释放那些 miss 的 BlockingCache 锁
        reset();                                             // 不写 cache、丢弃缓冲
    }
}
```

**`TransactionalCacheManager`**（`cache/TransactionalCacheManager.java`）维护 `Map<Cache, TransactionalCache>` —— 每个 L2 cache 第一次访问时包一层，**全局共享**：

```java
public void commit() {
    for (TransactionalCache txCache : transactionalCaches.values()) {
        txCache.commit();
    }
}
public Object getObject(Cache cache, CacheKey key) {
    return getTransactionalCache(cache).getObject(key);
}
```

**两步策略**：

1. 事务中所有 `putObject` 只入缓冲；`getObject` 仍直读 delegate（**但 commit 一发生就清 delegate**）
2. commit：先 `delegate.clear()` → `flushPendingEntries()` → `reset()`，**保证"事务读取"看到的就是"事务修改"**

### 5.5 L1 vs L2 数据流

```
BaseExecutor（L1 = PerpetualCache）
   ↑
CachingExecutor（L2 = tcm: TransactionalCacheManager）
   ↑
SqlSession.selectList()
```

| 缓存 | 实现 | 范围 | 失效 |
|---|---|---|---|
| L1 | `BaseExecutor.localCache: PerpetualCache` | 一次 SqlSession 期间 | commit/rollback/update；或 STATEMENT scope 出栈 |
| L2 | `CachingExecutor` 装饰的 `Transaction[Cache]` + `Cache` SPI | 跨 SqlSession（同一 Configuration） | namespace `clear()`、DML `flushCache=true`、commit 后 `clear()` |
| L2 事务缓冲 | `TransactionalCache` | 同一次事务 | commit (`flush`)、rollback (`discard`)、session close |

---

## 6. 拦截器（Plugin）

### 6.1 接口（`plugin/Interceptor.java`，35 行）

```java
public interface Interceptor {
    Object intercept(Invocation invocation) throws Throwable;
    default Object plugin(Object target) { return Plugin.wrap(target, this); }    // ★ 自动代理
    default void setProperties(Properties properties) { /* NOP */ }
}
```

### 6.2 注解（`Intercepts` + `Signature`）

```java
@Intercepts({
    @Signature(type = Executor.class, method = "query",
               args = {MappedStatement.class, Object.class, RowBounds.class, ResultHandler.class, CacheKey.class, BoundSql.class})
})
public class MyPlugin implements Interceptor { ... }
```

### 6.3 `Plugin.wrap`（`plugin/Plugin.java`，101 行）

**MyBatis 经典做法**：用 JDK 动态代理把 `Interceptor.pluginAll(target)` 包装成代理对象，并按 `@Signature.type` 决定是否拦截（拦截到的方法调用 `Invocation.proceed()` 进入原对象）。

注意：`Plugin.wrap` 只能拦截**接口方法**——`Executor` 接口的 17 个方法可以全打住，但想要拦截到 `BaseExecutor` 内部的方法得让用户自己用 `ResultSetHandler` / `StatementHandler` 等接口暴露点。

### 6.4 `Invocation`（`plugin/Invocation.java`，64 行）

```java
public class Invocation {
    private Object target;          // 被代理对象
    private Method method;
    private Object[] args;
    public Object proceed() throws InvocationTargetException, IllegalAccessException {
        return method.invoke(target, args);
    }
}
```

### 6.5 `InterceptorChain`

`Configuration.interceptorChain.pluginAll(target)` 把对象放进拦截器链条——装饰链如下：

```
target → wrap(executor, int1) → wrap(..., int2) → ... → 内层真实对象
```

---

## 7. Mapper 绑定（Binding）

### 7.1 链路总图

```
SqlSession.getMapper(Class)
   └─ MapperRegistry.getMapper(Class)
        └─ MapperProxyFactory.newInstance(sqlSession)
             └─ MapperProxy<T>           // JDK Proxy, implements InvocationHandler
                  └─ MapperMethod.execute(sqlSession, args)
                       └─ sqlSession.selectOne/insert/...  // 真打 SqlSession
```

### 7.2 `MapperProxy`（JDK 动态代理）

`invoke()` 中识别是不是 `Object#equals/toString/hashCode`，其它 method 都视为 mapper 调用 → `MapperMethod.execute(...)`。

### 7.3 `MapperMethod`（386 行）

- 构造：`new MapperMethod(Class<?> mapperInterface, Method method, Configuration config)` → 内部 `SqlCommand` + `MethodSignature`
- `SqlCommand`：根据 `Method.getAnnotation(Select.class)` 等推断 SQL 类型（`SqlCommandType`）
- `MethodSignature`：检测返回类型（`returnsList / returnsMap / returnsCursor / returnsOptional / ...`）和 `@Param` 名称
- `execute(sqlSession, args)` 是 switch over `SqlCommandType` 的 dispatch center（详见源代码 `mapper_method.execute`）

### 7.4 `MapperRegistry`（124 行）

`HashMap<Class<?>, MapperProxyFactory<?>>` 注册表，新 mapper class 首次出现时反射读 `@Mapper` 注解 + 解析 XML 关联。

---

## 8. 结果处理

### 8.1 参数解析（`reflection/ParamNameResolver.java`）

按 `@Param` 顺序 + 兜底加 `param1, param2, ...` 名称，再做单参数透传到 boundSql。

### 8.2 反射（`reflection/MetaObject.java` + `Reflector.java`）

- `MetaObject`：包装对象/Map，提供 `set/get/hasValue` Ognl-like 行为，但更严格——不解析子表达式（OGNL 留给 scripting 模块）
- `Reflector`：一次反射生成 setter/getter 索引缓存到 `Map<String, Invoker>`

### 8.3 TypeHandler（`type/`）

`TypeHandler<T>` 是泛型 SPI，每个 JDBC type ↔ Java type 对应一个实现。注册表 `TypeHandlerRegistry`，`MappedStatement.typeHandler` 与 `<typeHandler>` 配置可拓展。

### 8.4 ResultSetHandler（`executor/resultset/DefaultResultSetHandler.java`，~ 800 行）

最复杂的类：根据 `ResultMap` 解析 `ResultMapping`（含嵌套 association / collection），**触发嵌套查询**——一发现未填字段就 `deferLoad(...)` 加入延迟栈。

### 8.5 StatementHandler（`executor/statement/`）

四个实现：
- `SimpleStatementHandler`（无参 `Statement`）
- `PreparedStatementHandler`（预编译，最常用）
- `CallableStatementHandler`（存储过程）
- `RoutingStatementHandler`（按 `mappedStatement.statementType` 调度）

### 8.6 ResultLoader + Javassist/Cglib 代理（`executor/loader/`）

`@One` / `@Many` 标注的嵌套 association/collection 默认是 **嵌套 select** + **Javassist/Cglib** 生成代理 ResultMap 行为——访问属性时才发起查询。

---

## 9. 数据源

### 9.1 三种实现

| 文件 | 内容 |
|---|---|
| `datasource/jndi/JndiDataSourceFactory.java` | JndiDataSource（应用服务器查找） |
| `datasource/unpooled/UnpooledDataSource.java` | 直连 JDBC；每次 `getConnection()` 新连接 |
| `datasource/pooled/PooledDataSource.java` （644 行） | 经典连接池：空闲/活跃/等待队列 + maxActive + maxIdle |

### 9.2 事务（`transaction/`）

- `JdbcTransaction`（普通 JDBC commit/rollback）
- `ManagedTransaction`（由外部容器如 Spring）
- `TransactionFactory`：根据 `Environment.transactionManager` 选择

### 9.3 日志（`logging/`）

- 6 个日志 facade（Log/Slf4j/CommonsLog/Jdk14Log/StdOutImpl/NoLoggingImpl）
- `ConnectionLogger` + `StatementLogger` 通过 JDK 动态代理 hook JDBC

---

## 10. 数据流全链路 ASCII 流程图

```
              ┌─────────── user code ────────────┐
              │  @Select("SELECT ...")            │
              │  UserMapper mapper = session      │
              │          .getMapper(UserMapper    │
              │                    .class);      │
              └───────────────┬──────────────────┘
                              │
                              ▼
              ┌─── MapperProxy.invoke ───┐
              │    MapperMethod.execute  │
              └────────────┬─────────────┘
                             ▼
              ┌─── SqlSession.selectX ────┐       ───── ★ 关键 3 个 Executor ★
              │        (DefaultSqlSession) │
              └────────────┬──────────────┘
                            ▼
                  ┌── CachingExecutor? ──┐
                  │  if (ms.getCache())    │
                  │   tcm.get / delegate  │  ← ── L2 (Transaction-aware)
                  └────────────┬──────────┘
                               ▼ no cache
              ┌─── BaseExecutor ─────────┐
              │  localCache (L1) hit ?     │
              │  yes → return             │
              │  no  → queryFromDatabase │
              └────────────┬──────────────┘
                           ▼
              ┌─── SimpleExecutor.doQuery ─────┐
              │  RoutingStatementHandler        │
              │  PreparedStatementHandler       │
              │    ParameterHandler(参数绑定)   │
              │    Statement.execute()         │
              │  ResultSetHandler.handleRS     │
              │  DefaultResultHandler          │
              └────────────┬───────────────────┘
                           ▼
                  ┌── deferredLoads ──┐
                  │  if queryStack==0 │
                  │   load all deferred│
                  └─────────┬──────────┘
                            ▼
                    List<E> return
```

> 拦截器位置：`Configuration.interceptorChain.pluginAll(executor)` 在 SqlSession 创建时就包好，对 L1/L2 一视同仁。

---

## 11. 与 rbatis / rbatis-cache 对照速记表

| 维度 | MyBatis 3 | rbatis (workspace-easy-4-rust) | rbatis-cache |
|---|---|---|---|
| 模型 | 反射 + 注解/XML + JDK 动态代理 | 过程宏 + 编译期 AST | SPI + 字节级多 backend |
| SQL DSL | XML 标签树 / `@Select(...)` | py_sql / html_sql（类 Python / 类 XML） | 无（value 是字节流） |
| Executor trait | 17 个方法的接口 | `Executor` trait 5 个方法 | 通过 `CacheInterceptor::get_or_load` |
| 拦截器 | `Interceptor` + `Plugin.wrap` JDK 代理 | `Intercept` trait + `apply_before/after` | 也是 `Interceptor` 模式 |
| 缓存键 | `CacheKey = list<Object> + checksum + hashcode` | `CacheKey { namespace, sql, args, version, digest: u128 }`（xxh3-128） | `CacheKey` + 8 维隔离边界（BLAKE3 长度前缀） |
| L1 实现 | `BaseExecutor.localCache: PerpetualCache` | 同步实现 → 重写为分片 `L1Cache(executor → shard, Arc<Value>, capacity + TTL)` | 不需要 L1（进程外） |
| L2 实现 | 用户实现 `Cache` SPI + 11 个装饰器 + `TransactionalCacheManager` | `MemoryCacheStore`（单 moka + weigher + Expiry） | `LocalBackend` / `RedisCacheBackend` / `MemcachedCacheBackend` |
| L2 事务 | `TransactionalCache` (commit-flush / rollback-discard) | 同步设计 → `TransactionCacheMode::{Bypass, Defer}` | `get_or_load` 一律 `in_transaction=true` 旁路 |
| Singleflight | `BlockingCache`（`CountDownLatch` + `ConcurrentHashMap`） | 自实现 `Arc<Notify>` follower leader | 自实现 `Arc<Mutex>` + `strong_count==2` 启发式 |
| generation 失效 | 通过 `Cache.clear()`（整个实例） | 通过 epoch + store key | `bump_generation(ns)` + envelope `generation` 字段 |
| 大对象保护 | 无强制上限 | `max_value_size` 跳过 L2 | `CachePolicy::max_value_size` |
| 框架注解 | 30+ `@Select/@Update/@Insert/@Delete/@Param` | 无对应（宏代替） | 无 |
| 流式结果 | `Cursor<T>`（底层 `ResultSet`） | 同步回调 | 不涉及（payload 都是 `Vec<u8>`） |

> 一句话：MyBatis 是"反射驱动的 SQL mapper + JDK 代理拦截链"；rbatis 把反射改为过程宏 + 把代理改为 trait 调色；rbatis-cache 把内存对象改成字节 envelope 适配多 backend。

---

## 12. 关键设计权衡（FAQ）

### Q1：为什么 `CacheKey` 用"列表 + 多重 equals" 而不直接用 hash digest？

为了让不同 JDBC 驱动/不同 rowBounds/不同 SQL id 拼装时能 step-by-step 累加，hashcode + checksum + count 三段预过滤，**绝大多数不同 key 在 hashcode 不等时直接 short-circuit**——避免数组逐元素 equals。这对 Java 大对象而言比"先拼再算 digest"更细粒度。

### Q2：为什么 `BlockingCache` 在装饰链最外层？

`BlockingCache.getObject(key)` 是"先抢锁、读、命中放锁"的语义，**它必须在最外**；否则 `LoggingCache`、`LruCache` 等把 key 转换掉，锁与原 key 对不上。

### Q3：为什么 `CachingExecutor.close(forceRollback)` 要在 finally 里 delegate.close？

issue #499/#524/#573：之前未 finally，会出现 close 抛异常导致 tcm 既没 commit 也没 rollback。改为 finally 保证两层都关。

### Q4：`Cache.removeObject` 看起来是删除但实际语义是"释放锁"？

来自历史：`BlockingCache` 当年需要"显式释放"——但 `getObject` 命中时也会自动 release。**rollback 路径没人调用 getObject**，所以 `removeObject` 现在专门为 rollback 服务。文档注释清楚写道："despite its name, this method is called only to release locks"。

### Q5：为什么 `CachingExecutor.tcm` 独立于 `SqlSession`？

`tcm` 是 **`Configuration` 级别**的（`CachingExecutor` 持有），所以跨 SqlSession 共享 L2，但**缓存的写入时机还是事务边界**——`TransactionalCache` 的 commit/rollback 由 SqlSession close 时触发。

### Q6：为什么 `LocalCacheScope.STATEMENT` 在 `queryStack==0` 出栈清？

issue #482：当用户启用 `STATEMENT` 作用域，希望"该 statement 结束后立刻 clean L1"——出栈时不依赖 session 关闭。**这是嵌套查询的特殊情况**——保证外层查询完成时所有嵌套的占位 key 都清干净。

### Q7：`@CacheNamespace` 与 `<cache/>` XML 怎么关联？

通过 `MapperBuilderAssistant.useNewCache(...)` → `Configuration.addCache(Cache)`；`MappedStatement.cache` 通过 `XMLMapperBuilder.processCacheRef(...)` 解析 `@CacheNamespaceRef` 跨 mapper 共享同一个 Cache 实例。

### Q8：为什么 `deferLoad` 要跟外层 query 出栈结合？

避免在 `ResultSet.next()` 循环中触发嵌套查询（因为 resultset 在 query 没关闭前不允许其他操作）。把所有 `DeferredLoad` 推到一个队列，外层 `queryStack==0` 时再批量 `load()`——这是经典 write-behind。

---

## 13. codegraph 速查命令

（已索引：1,396 Java 文件 + 441 xml + 9 yaml + 7 properties）

```bash
export PATH="/Users/wandl/.nvm/versions/node/v24.18.0/bin:$PATH"
cd /Users/wandl/workspaces/workspace-github/mybatis-3

codegraph status
codegraph query "SqlSession\|SqlSessionFactory"
codegraph query "Executor\|BaseExecutor\|CachingExecutor\|SimpleExecutor\|ReuseExecutor\|BatchExecutor"
codegraph query "Cache\|CacheKey\|TransactionalCache\|BlockingCache\|LruCache\|FifoCache"
codegraph query "Interceptor\|Plugin\|Invocation\|Intercepts\|Signature"
codegraph query "MappedStatement\|BoundSql\|SqlSource"
codegraph query "MapperProxy\|MapperRegistry\|MapperMethod"
codegraph query "MetaObject\|Reflector\|ParamNameResolver"
codegraph query "TypeHandler\|TypeHandlerRegistry"
codegraph query "ResultSetHandler\|StatementHandler\|ParameterHandler"
```

---

## 14. 推荐阅读顺序

1. `pom.xml`（依赖：`ognl` / `javassist` / `cglib`）——看运行时强依赖
2. `session/SqlSession.java` + `session/defaults/DefaultSqlSession.java`
3. `session/Configuration.java`（**中心容器**——1200 行可只读头部字段定义）
4. `mapping/MappedStatement.java` + `mapping/BoundSql.java`
5. `executor/Executor.java` + `BaseExecutor.java` + `SimpleExecutor.java`
6. `executor/CachingExecutor.java`（二级缓存入口）
7. `cache/Cache.java` + `cache/CacheKey.java`
8. `cache/decorators/TransactionalCache.java` + `TransactionalCacheManager.java`（**L2 事务关键**）
9. `cache/decorators/BlockingCache.java`（单飞实现）
10. `plugin/Interceptor.java` + `Plugin.java` + `Invocation.java`
11. `binding/MapperProxy.java` + `MapperMethod.java` + `MapperRegistry.java`
12. `builder/xml/XMLConfigBuilder.java` → `XMLMapperBuilder.java` → `XMLStatementBuilder.java`（自顶向下）
13. `executor/resultset/DefaultResultSetHandler.java`（阅读难点，**留到后期**）

---

## 15. 未变 / 已废弃 / 注意点

1. **`Cache.getReadWriteLock()` 自 3.2.6 起已 noop**——文档注释："As of 3.2.6 this method is no longer called by the core."（注释在 `cache/Cache.java:91`），所以这里所有 cache 适配器的 `getReadWriteLock()` 都可以不写。
2. **`ParameterMap` 已淡化**：MyBatis 文档里推荐"用 `@Param` 替代 ParameterMap"；Configuration 里 parameterMaps 仍存在但几乎不再用。
3. **`XMLConfigBuilder.propertiesElement(...)`** 支持外部属性，但**二级缓存配置文件里的 `<cache size="..."/>` 之外**,真实 memory caching 还需用 `LruCache`/`PerpetualCache`。
4. **`@CacheNamespace` 默认 `flushInterval` 与 `BlockCache` 协同**：`<cache eviction="LRU" size="1024" type="BlockingCache"/>` 是最常见组合。
5. **OGNL 注入**：3.x 仍然用 OGNL 3.4.11 处理 `<if test="...">`；注意表达式内不要直接拼接用户输入。
6. **`ExecutorType.BATCH`** 与 `flushStatements()` 必须手动调用——否则批量的 `update` 只入队不入 DB，session 关闭也会自动 flush。
7. **L2 与 connection 复用无关**：L2 生命周期由 session 关闭驱动 commit，而不是 Connection 关闭。这意味着**同一个 SqlSession 复用一条 Connection**时 L2 不会自动清——这是 `localCacheScope=STATEMENT` 与 `flushCacheRequired` 的存在意义。
8. **拦截器对 `Statement`/`ResultSet` 屏蔽**：`plugin/Plugin.java:101` 不能拦截接口外方法——若要 hack StatementHandler 的内部行为只能改源码或用 wrap。

---

## 附：`executor/loader` 延迟加载选择

| 文件 | 用途 |
|---|---|
| `executor/loader/AbstractEnhancedDeserializationProxy.java` | 序列化延迟对象 |
| `executor/loader/AbstractSerialStateHolder.java` | 持久化状态保存 |
| `executor/loader/CglibProxyFactory.java` + `cglib/*` | CGLib 字节码生成代理 |
| `executor/loader/JavassistProxyFactory.java` + `javassist/*` | Javassist 生成代理（默认） |
| `executor/loader/ProxyFactory.java` | `ProxyFactory SPI — selectById/...` |
| `executor/loader/ResultLoader.java` | 单个延迟加载的执行单元 |
| `executor/loader/ResultLoaderMap.java` | 维护对象图上的延迟字段索引 |
| `executor/loader/WriteReplaceInterface.java` | 序列化兼容接口 |

`ProxyFactory` 通过 `Configuration.proxyFactory = "CGLIB"`/`"JAVASSIST"` 切换。

---

如果你是要把 rbatis / rbatis-cache 与 MyBatis 对照演进，本仓库是标杆答案；如果是要给 rbatis 加 `@CacheNamespace` 风格的注解映射，关键三处是：
- `Builder.useNewCache(...)`
- `Configuration.addCache(Cache)`
- `MappedStatement.cache`

这与 `rbatis/src/plugin/cache/CachePolicy { namespace, ... }` 之间是一一对应的。

---

## 相关阅读（rbatis-plus 生态）

**本目录 `rbatis-plus/docs/` 是给"按 MyBatis-Plus 设计思路做 Rust 移植"项目准备的：**

- [`../mybatis-plus-architecture.md`](../mybatis-plus-architecture.md) — **MyBatis-Plus（Service 层 / CRUD 模板 / Wrapper 链 / InnerInterceptor 体系）**
  - §3 `BaseMapper<T>` 与 Rust trait + `crud!{}` 宏的对位
  - §7 `MybatisPlusInterceptor` 的"二次分发"模型
  - §8 乐观锁 + 分页 13 dialect 的实现
  - 附录 B：6 维度 Rust 移植 checklist
- `../../rbatis/docs/rbatis-architecture.md` — **rbatis（Rust 主仓库，已合入 Caffeine L2 缓存 `df87ac41`）**
- `../../rbatis-cache/docs/rbatis-cache-architecture.md` — **rbatis-cache（多 backend 字节级缓存 SPI）**

> **建议按下面的顺序读这三份：**
>
> 1. `rbatis-architecture.md`（先懂 rbatis 自身：Executor trait、Intercept、缓存生命周期）
> 2. **本文**（本文件）`mybatis-3-architecture.md`（MyBatis 3 主流程）
> 3. **`mybatis-plus-architecture.md`**（MyBatis-Plus 在 MyBatis 之上做了什么）
