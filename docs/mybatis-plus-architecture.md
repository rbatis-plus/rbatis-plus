# MyBatis-Plus 架构与代码导读

> 本文档基于本地 **codegraph** 索引（`/Users/wandl/workspaces/workspace-github/mybatis-plus`，956 个 Java 文件 + 45 xml + 42 kotlin + 12 yaml）的一手源码梳理。
>
> 仓库根命名：`**com.baomidou**`（苞米豆）；在 MyBatis 之上扩展，是国内最流行的 MyBatis 增强工具包。
>
> 参考资料：
> - GitHub：<https://github.com/baomidou/mybatis-plus>
> - 本地版本：`MyBatis-Plus 3.5.17`（2026.07.07 release，源自 `CHANGELOG.md`）

---

## 目录

1. 一句话定位与设计哲学
2. 多模块仓库布局与规模
3. **`BaseMapper<T>`：CRUD 不写一行 SQL**
4. **`IService<T>` + `ServiceImpl<M, T>`：Service 层**
5. **SQL 注入器（SqlInjector）**
6. **Wrapper（条件构造器）：Lambda 式 SQL**
7. **插件体系（MybatisPlusInterceptor + InnerInterceptor）**
8. **核心插件：分页 / 乐观锁 / 多租户 / 防全表更新**
9. **`TableInfo`：反射元数据**
10. **数据流 ASCII 流程图**
11. **与 MyBatis 3 / rbatis / rbatis-cache 对照速记表**
12. **关键设计权衡（FAQ）**
13. **codegraph 速查命令**
14. **推荐阅读顺序**
15. **未变 / 已废弃 / 注意点**

---

## 1. 一句话定位与设计哲学

**MyBatis-Plus = MyBatis 增强，不是替代。** 仅改 MyBatis：

> "为简化开发、提高效率而生"（来自 README）

它做了三件事：

1. **消灭"CRUD 样板"**：`BaseMapper<T>` 接口继承即可获得 30+ 通用方法
2. **消灭"单表 Wrapper XML"**：`QueryWrapper<T>` / `LambdaQueryWrapper<T>` / `UpdateWrapper<T>` 让构建条件像"链式调用 Map"
3. **消灭"通用拦截需求空白"**：分页 / 乐观锁 / 多租户 / 数据权限 / 防全表更新 / SQL 性能规范 — 都做成 **InnerInterceptor**

### 1.1 与同类对比速读

| 维度 | MyBatis-Plus | rbatis | rbatis-cache |
|---|---|---|---|
| 模型 | MyBatis 反射+注解 加 Service/Mapper 层工具 | 过程宏 + trait 拦截链 | 字节级多 backend SPI |
| 模板生成 | `BaseMapper<T>` 接口继承（Java 反射约定） | `crud!(T {})` 宏展开（编译期） | 不涉及（SPI） |
| 条件构造 | `LambdaQueryWrapper<T>` 流式 API | `value!{"col":"x","col2":["a","b"]}` value map | 不涉及 |
| 拦截器 | `MybatisPlusInterceptor`（JDK 代理）+ InnerInterceptor | `Intercept::before/after` | `get_or_load` |
| 乐观锁 / 多租户 / 分页 | 都是 InnerInterceptor | 手动 + 自实现 | 不涉及 |

---

## 2. 多模块仓库布局与规模

```
mybatis-plus/                                          Gradle root（已废弃 pom 文件）
├── build.gradle                                   ── Kotlin DSL，约 200 行
├── gradle/libs.versions.toml（依赖版本目录）        ── 所有 lib.* 引用
├── mybatis-plus/                                  ── 顶层聚合（已无 main/java 源码）
├── mybatis-plus-annotation/                       ── 16 个注解                        独立 jar
│   └── src/main/java/com/baomidou/mybatisplus/annotation/
│       ├── DbType.java FieldFill.java FieldStrategy.java
│       ├── IEnum.java IdType.java InterceptorIgnore.java
│       ├── KeySequence.java OrderBy.java
│       ├── SqlCondition.java TableField.java TableId.java
│       ├── TableLogic.java TableName.java Version.java EnumValue.java
│       └── package-info.java
├── mybatis-plus-core/                              ── ★ 核心实现 ★  170 个 main 源 java
│   └── src/main/java/com/baomidou/mybatisplus/core/
│       ├── batch/                                 ── BatchSqlSession / MybatisBatch
│       ├── config/                                ── GlobalConfig + DbConfig
│       ├── conditions/                            ── Wrapper 体系（核心 5 个文件 + 9 子包）
│       ├── enums/                                 ── SqlMethod / SqlKeyword / ...
│       ├── exceptions/                            ── MybatisPlusException
│       ├── handlers/                              ── MetaObjectHandler（自动填充）
│       ├── incrementer/                           ── 主键生成器 SPI（KeyGenerator 子接口）
│       ├── injector/                              ── ★ SQL 注入器 ★
│       │   ├── AbstractSqlInjector.java           ── 串联方法表的 base class
│       │   ├── AbstractMethod.java                ── 每方法类继承的 base
│       │   ├── DefaultSqlInjector.java            ── 默认注入 14 个方法（含 PK 判定）
│       │   └── methods/                           ── Insert / Update / Delete / Select*
│       ├── mapper/                                ── BaseMapper / Mapper / MapperProxyMetadata
│       ├── metadata/                              ── TableInfo / TableFieldInfo / IPage
│       ├── override/                              ── MybatisMapperMethod（替换 MyBatis）
│       ├── plugins/                               ── InterceptorIgnoreHelper
│       ├── spi/                                   ── SqlInjector / KeyGenerator SPI
│       └── toolkit/                               ── 工具集（反射、SQL 拼接、SqlScriptUtils、StringPool）
├── mybatis-plus-extension/                         ── ★ 扩展 ★（8 内置 InnerInterceptor）
├── mybatis-plus-spring/                            ── `ServiceImpl<M, T>` 实现类 + `IService`
├── mybatis-plus-jsqlparser-support/                  ── 内置 sqlparser 多版本适配
│   ├── mybatis-plus-jsqlparser-4.9/                ── 内含 PaginationInnerInterceptor 478 行
│   ├── mybatis-plus-jsqlparser-5.0/                同上（jsqlparser 5）
│   └── mybatis-plus-jsqlparser/                    同上（jsqlparser latest）
├── mybatis-plus-generator/                          ── ★ 代码生成器 ★
│   └── src/main/java/com/baomidou/mybatisplus/generator/
├── mybatis-plus-bom/                                ── BOM
└── spring-boot-starter/                             ── Spring Boot starter
```

codegraph 总览：

```
Files:     1056   (Java 956, Kotlin 42, xml 45, yaml 12, properties 3)
DB Size:   ~ 12 MB（注意本仓库有额外的 `.code-review-graph` 仓内索引，约 1.6MB）
```

---

## 3. `BaseMapper<T>`：CRUD 不写一行 SQL

### 3.1 接口一览（`mybatis-plus-core/.../core/mapper/BaseMapper.java`，624 行，泛型 ID）

继承 `Mapper<T>`，**默认接口** + **抽象方法** 共 30+。源码关键示例（节选）：

```java
public interface BaseMapper<T> extends Mapper<T> {

    int insert(T entity);

    default int deleteById(Serializable id) { return deleteById(id, true); }
    int deleteById(T entity);
    default int deleteByMap(Map<String, Object> columnMap) { return delete(Wrappers.<T>query().allEq(columnMap)); }
    int delete(@Param(Constants.WRAPPER) Wrapper<T> queryWrapper);
    @Deprecated default int deleteBatchIds(@Param(Constants.COLL) Collection<?> idList) { return deleteByIds(idList); }
    default int deleteByIds(Collection<?> collections, boolean useFill) { ... }   // issue-578 fix

    int updateById(@Param(Constants.ENTITY) T entity);
    int update(@Param(Constants.ENTITY) T entity, @Param(Constants.WRAPPER) Wrapper<T> updateWrapper);
    default int update(Wrapper<T> uw) { return update(null, uw); }

    T selectById(Serializable id);
    List<T> selectByIds(Collection<? extends Serializable> idList);
    Cursor<T> selectWithCursor(Wrapper<T> queryWrapper);
    Long selectCount(Wrapper<T> queryWrapper);
    List<T> selectList(Wrapper<T> queryWrapper);
    List<Map<String, Object>> selectMaps(Wrapper<T> queryWrapper);
    <E> List<E> selectObjs(Wrapper<T> queryWrapper);
    <P extends IPage<T>> P selectPage(P page, Wrapper<T> queryWrapper);
    boolean exists(Wrapper<T> queryWrapper);
    default boolean insertOrUpdate(T entity) { ... }
    default List<BatchResult> insert(Collection<T> entityList) { ... }
    default List<BatchResult> updateById(Collection<T> entityList) { ... }
    ...
}
```

### 3.2 关键设计模式：Mapper 上的 default method 直接调 `SqlSession`

观察 `default int deleteByIds(Collection<?>, boolean useFill)`：

```java
default int deleteByIds(Collection<?> collections, boolean useFill) {
    if (CollectionUtils.isEmpty(collections)) return 0;
    MapperProxyMetadata md = MybatisUtils.getMapperProxy(this);    // ★ 取出（this = Mapper 代理）
    SqlSession sqlSession = md.getSqlSession();
    Class<?> mapperInterface = md.getMapperInterface();
    TableInfo tableInfo = TableInfoHelper.getTableInfo(entityClass);
    Map<String, Object> params = new HashMap<>();
    if (useFill && tableInfo.isWithLogicDelete() && tableInfo.isWithUpdateFill()) {
        params.put(Constants.MP_FILL_ET, tableInfo.newInstance());
    }
    params.put(Constants.COLL, collections);
    return sqlSession.delete(
        mapperInterface.getName() + StringPool.DOT + SqlMethod.DELETE_BY_IDS.getMethod(),
        params
    );
}
```

- **方法名 `deleteByIds` 拼成 SQL ID = `<Mapper>.deleteByIds`**，与 `MappedStatement` 一一对应
- 底层还是 `sqlSession.delete(id, params)` —— **走 MyBatis 缓存与拦截器链**，没有任何"独立执行通道"
- `TableInfoHelper.getTableInfo(entityClass)` 拿到表元数据 → 用于逻辑删除填充等

### 3.3 与 `Mapper<T>` 的关系

```java
// mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/mapper/Mapper.java
public interface Mapper<T> {
    // 仅作标记；用于泛型推导
    String TABLE_NAME = "";  // 官方推荐继承 BaseMapper，此常量无意义；保留向后兼容
}
```

### 3.4 自定义 mapper

用户只需要：

```java
@Mapper
public interface UserMapper extends BaseMapper<User> {
    // 自定义复杂 SQL 自己加 @Select / @Update 或 XML
    @Select("SELECT * FROM user WHERE id = #{id}")
    User findById(Long id);
}
```

—— 自定义 SQL 与通用方法 **共存**。这是 MyBatis-Plus 不同于"全 ORM"的最关键设计点。

---

## 4. `IService<T>` + `ServiceImpl<M, T>`：Service 层

### 4.1 接口（`mybatis-plus-spring/.../service/IService.java`，75 行）

`IService<T>` 是 Spring 模块的"顶级 Service"，**通过 IRepository<T> 桥接到主仓**，未单独展开：

```java
public interface IService<T> extends IRepository<T> {
    @Transactional(rollbackFor = Exception.class)
    default boolean saveBatch(Collection<T> entityList) { return saveBatch(entityList, DEFAULT_BATCH_SIZE); }
    default boolean saveOrUpdateBatch(Collection<T> entityList) { ... }
    default boolean removeBatchByIds(Collection<?> list) { return removeByIds(list); }
    default boolean updateBatchById(Collection<T> entityList) { ... }
}
```

### 4.2 实现（`ServiceImpl<M, T>`）

`ServiceImpl<M extends BaseMapper<T>, T>` 内部：

```java
public class ServiceImpl<M extends BaseMapper<T>, T> implements IService<T> {
    @Autowired
    protected M baseMapper;        // ★ 注入 mapper
    // 实现方法直接委托给 baseMapper.insert/update/...
}
```

每次方法调用最终回到 BaseMapper。

### 4.3 `IRepository` 是 `extension` 模块的纯接口

`mybatis-plus-extension/.../extension/repository/IRepository.java` + `AbstractRepository.java`：

- 提供非 Spring 环境下的 Repository
- `IService` 继承自 `IRepository` 而非直接实现，**让 IService 不绑定 Spring 容器**

---

## 5. SQL 注入器（SqlInjector）

### 5.1 注册路径

`Configuration` 在 `Configuration#addMapper(...)` 时（MyBatis 流程）调用 `MapperRegistry`：

1. `MapperRegistry.addMapper(...)` → 检查 mapper 继承 `BaseMapper` ？
2. 若是 → 调 `GlobalConfigUtils.getSqlInjector(configuration).inspectInject(...)` 把当前 `DefaultSqlInjector` 注册上
3. `SqlInjector.inspectInject(...)` → 拿到所有 `AbstractMethod`（Insert/Delete/Update/Select*） → `addMappedStatement(...)` 一个一个注册

### 5.2 `DefaultSqlInjector`（42 行 + 13 个 method 文件）

```java
public class DefaultSqlInjector extends AbstractSqlInjector {
    @Override
    public List<AbstractMethod> getMethodList(Configuration configuration, Class<?> mapperClass, TableInfo tableInfo) {
        GlobalConfig.DbConfig dbConfig = GlobalConfigUtils.getDbConfig(configuration);
        Stream.Builder<AbstractMethod> builder = Stream.<AbstractMethod>builder()
            .add(new Insert(dbConfig.isInsertIgnoreAutoIncrementColumn()))
            .add(new Delete())                                              // delete(W)
            .add(new Update())                                              // update(W)
            .add(new SelectCount())
            .add(new SelectMaps())
            .add(new SelectObjs())
            .add(new SelectList())
            .add(new SelectWithCursor());                                   // 8 个无 PK 方法
        if (tableInfo.havePK()) {
            builder.add(new DeleteById())
                .add(new DeleteByIds())
                .add(new UpdateById())
                .add(new SelectById())
                .add(new SelectByIds());                                    // +5 个含 PK 方法
        } else {
            logger.warn(tableInfo.getEntityType() + " Not found @TableId annotation, Cannot use Mybatis-Plus 'xxById' Method.");
        }
        return builder.build().collect(toList());
    }
}
```

### 5.3 `AbstractMethod` + `methods/*.java` 单方法文件

每个方法独立文件，**核心 4 行**：

```java
public class UpdateById extends AbstractMethod {
    @Override
    public MappedStatement injectMappedStatement(Class<?> mapperClass, Class<?> modelClass, TableInfo tableInfo) {
        final String additional = optlockVersion(tableInfo) + tableInfo.getLogicDeleteSql(true, true);
        String sql = SqlMethod.UPDATE_BY_ID.format(
            tableInfo.getTableName(),
            sqlSet(tableInfo.isWithLogicDelete(), false, tableInfo, false, ENTITY, ENTITY_DOT),
            tableInfo.getKeyColumn(),
            ENTITY_DOT + tableInfo.getKeyProperty(),
            additional);
        SqlSource sqlSource = super.createSqlSource(configuration, sql, modelClass);
        return addUpdateMappedStatement(mapperClass, modelClass, methodName, sqlSource);
    }
}
```

模板字符串 `SqlMethod.UPDATE_BY_ID.format(...)`：

```
UPDATE %s SET %s %s = %s %s
```

参数替换得到具体 SQL，例如：

```
UPDATE user SET name=#{et.name},version=2 , id=#{et.id},version=1
where id=? AND version=1 AND deleted=0
```

### 5.4 `SqlMethod` 枚举常量

`core/enums/SqlMethod.java` 集中所有 14+ 模板字符串：

```java
public enum SqlMethod {
    INSERT_ONE("insert", "插入一条数据（选择字段插入）",
        "<script>INSERT INTO %s %s VALUES %s %s</script>"),
    DELETE_BY_ID("deleteById", "根据 ID 删除", "DELETE FROM %s WHERE %s = %s"),
    UPDATE_BY_ID("updateById", "根据 ID 更新",
        "<script>UPDATE %s SET %s %s = %s %s %s</script>"),
    SELECT_LIST("selectList", "查询全部记录",
        "<script>SELECT %s FROM %s %s %s</script>"),
    ...
}
```

> 注：`<script>` 是 MyBatis 自带的 dynamic-SQL 标签，等价于 rbatis 的 `html_sql` 宏触发的边界。

---

## 6. Wrapper（条件构造器）：Lambda 式 SQL

### 6.1 类图

```
Wrapper<T>
  └── AbstractWrapper<T,R,Children>                    ── 通用方法 eq/ne/like/in/orderBy/groupBy/having...
        ├── QueryWrapper<T>                          ── SQL 字符串字段名
        ├── LambdaQueryWrapper<T>                  ── Field 方法引用（SFunction<T, ?>)
        └── UpdateWrapper<T>                        ── set(...) 加 update-by-wrapper
              └── LambdaUpdateWrapper<T>
```

### 6.2 用法示例

```java
// 1. LambdaQuery
List<User> users = userMapper.selectList(
    Wrappers.<User>lambdaQuery()
        .eq(User::getStatus, 1)
        .like(User::getName, "foo")
        .gt(User::getCreateTime, since)
        .orderByDesc(User::getId)
);

// 2. Update wrapper + Lambda
boolean ok = userMapper.update(null,
    Wrappers.<User>lambdaUpdate()
        .set(User::getStatus, 0)              // SET status = 0
        .eq(User::getId, 100L));

// 3. Condition 模式（条件构造）
List<User> active = userMapper.selectList(
    Wrappers.<User>lambdaQuery()
        .eq(User::getStatus, 1)
        .and(w -> w.like(User::getName, "x").or().like(User::getNick, "x")));
```

### 6.3 `AbstractWrapper` 的 5 个核心接口

`AbstractWrapper` 实现：

| interface | 能力 |
|---|---|
| `Compare<Children, R>` | eq/ne/gt/ge/lt/le/between/notBetween/... |
| `Nested<Children, Children>` | `and(...)` / `or(...)` / `nested(...)` |
| `Join<Children>` | `leftJoin / rightJoin / innerJoin / apply(...)` |
| `Func<Children, R>` | isNull/isNotNull/in/notIn/groupBy/orderByAsc/Desc/having |
| 内置 on  | like/.../sum/avg/min/max/... |

### 6.4 SQL 渲染：抽象 WHERE → `MergeSegments` → 字符串

每个 `eq / ne / like` 调用把一个 `ISqlSegment`（含 SQL 关键字 + 列名占位 + 参数占位）压入 `NormalSegmentList`；渲染时通过 `MergeSegments` 合并成最终的 `WHERE ... AND ... OR ...` 字符串。

```java
public abstract class AbstractWrapper<T, R, Children extends AbstractWrapper<T, R, Children>>
    extends Wrapper<T>
    implements Compare<Children, R>, Nested<Children, Children>, Join<Children>, Func<Children, R> {
    protected final Children typedThis = (Children) this;
    protected AtomicInteger paramNameSeq;     // 给 #{paramX} 占位发号
    ...
    public Children eq(boolean condition, R column, Object val) {
        return doEq(condition, column, val, SqlKeyword.EQ);
    }
}
```

### 6.5 SQL 注入防护

`toolkit/sql/SqlInjectionUtils.java`：在 INSERT/UPDATE 模板构造列名时检查 + 过滤参数 `it.hasText(...)` 风险字符。

---

## 7. 插件体系（MybatisPlusInterceptor + InnerInterceptor）

### 7.1 总架构（`mybatis-plus-extension/.../plugins/MybatisPlusInterceptor.java`，156 行）

```java
@Intercepts({
    @Signature(type = StatementHandler.class, method = "prepare",    args = {Connection.class, Integer.class}),
    @Signature(type = StatementHandler.class, method = "getBoundSql", args = {}),
    @Signature(type = Executor.class,         method = "update",      args = {MappedStatement.class, Object.class}),
    @Signature(type = Executor.class,         method = "query",      args = {MappedStatement.class, Object.class, RowBounds.class, ResultHandler.class}),
    @Signature(type = Executor.class,         method = "query",      args = {MappedStatement.class, Object.class, RowBounds.class, ResultHandler.class, CacheKey.class, BoundSql.class}),
})
public class MybatisPlusInterceptor implements Interceptor {
    @Setter private List<InnerInterceptor> interceptors = new ArrayList<>();
    ...
}
```

**与 MyBatis 原生 `InterceptorChain` 关系**：MyBatis-Plus 的 `MybatisPlusInterceptor` 是**一个** MyBatis `Interceptor` 实例，里面**包了一组** `InnerInterceptor`——它的 `intercept(Invocation)` 是统一的"分发器"。`Plugin.wrap(...)`（MyBatis 自带）会把 `Executor`/`StatementHandler` 包成 JDK 代理，统一入口到 `intercept(...)`。

### 7.2 `intercept(Invocation)` 完整分发逻辑

```java
public Object intercept(Invocation invocation) throws Throwable {
    Object target = invocation.getTarget();
    Object[] args = invocation.getArgs();
    if (target instanceof Executor) {
        final Executor executor = (Executor) target;
        Object parameter = args[1];
        boolean isUpdate = args.length == 2;
        MappedStatement ms = (MappedStatement) args[0];
        if (!isUpdate && ms.getSqlCommandType() == SqlCommandType.SELECT) {
            RowBounds rowBounds = (RowBounds) args[2];
            ResultHandler resultHandler = (ResultHandler) args[3];
            BoundSql boundSql;
            if (args.length == 4) {
                boundSql = ms.getBoundSql(parameter);
            } else {
                boundSql = (BoundSql) args[5];                 // 当被代理再调用
            }
            for (InnerInterceptor query : interceptors) {
                if (!query.willDoQuery(executor, ms, parameter, rowBounds, resultHandler, boundSql)) {
                    return Collections.emptyList();           // 短路返回空集合
                }
                query.beforeQuery(executor, ms, parameter, rowBounds, resultHandler, boundSql);
            }
            CacheKey cacheKey = executor.createCacheKey(ms, parameter, rowBounds, boundSql);
            return executor.query(ms, parameter, rowBounds, resultHandler, cacheKey, boundSql);   // ★ 自己再调一次 query，触发 CachingExecutor
        } else if (isUpdate) {
            for (InnerInterceptor update : interceptors) {
                if (!update.willDoUpdate(executor, ms, parameter)) {
                    return -1;
                }
                update.beforeUpdate(executor, ms, parameter);
            }
        }
    } else {
        // StatementHandler
        final StatementHandler sh = (StatementHandler) target;
        if (null == args) {
            // getBoundSql() 只有一个参数，无 args
            for (InnerInterceptor innerInterceptor : interceptors) {
                innerInterceptor.beforeGetBoundSql(sh);
            }
        } else {
            // prepare(connection, transactionTimeout)
            Connection connection = (Connection) args[0];
            Integer transactionTimeout = (Integer) args[1];
            for (InnerInterceptor innerInterceptor : interceptors) {
                innerInterceptor.beforePrepare(sh, connection, transactionTimeout);
            }
        }
    }
    return invocation.proceed();                            // 不属于匹配调度，则调原方法
}
```

**核心点**：
- **拦截点只有 5 个**（prepare / getBoundSql / update / query + cacheKey variants）；其它方法（如 commit/rollback/flushStatements）**不拦截**
- 通过 arg 数量区分 query 的两种重载（4 args vs 6 args）
- **再调一次 `executor.query(...)`**：让分页 / 多租户里的"调过 setParameter 后重新生成的 boundSql"成功被 CachingExecutor 缓存
- `willDoQuery/Update` 短路 `false` → 直接 `Collections.emptyList()`

### 7.3 `InnerInterceptor` SPI（`.../plugins/inner/InnerInterceptor.java`，127 行）

```java
public interface InnerInterceptor {
    default boolean willDoQuery(Executor, MappedStatement, Object parameter,
                                RowBounds, ResultHandler, BoundSql) throws SQLException { return true; }
    default void beforeQuery(Executor, MappedStatement, Object parameter,
                             RowBounds, ResultHandler, BoundSql) throws SQLException { }
    default boolean willDoUpdate(Executor, MappedStatement, Object parameter) throws SQLException { return true; }
    default void beforeUpdate(Executor, MappedStatement, Object parameter) throws SQLException { }
    default void beforePrepare(StatementHandler sh, Connection connection, Integer transactionTimeout) { }
    default void beforeGetBoundSql(StatementHandler sh) { }
    default void setProperties(Properties properties) { }
}
```

### 7.4 `plugin(Object target)` 选择性 wrap

```java
public Object plugin(Object target) {
    if (target instanceof Executor || target instanceof StatementHandler) {
        return Plugin.wrap(target, this);
    }
    return target;
}
```

——只包 Executor 和 StatementHandler。ParameterHandler / ResultSetHandler 不让 MyBatis-Plus 拦截。

### 7.5 配置属性：`@page` / `page:limit` 自描述协议

源码注释里明确：

```properties
# key: "@page", value: "com.baomidou...PaginationInnerInterceptor"
# key: "page:limit", value: "100"
```

`MybatisPlusInterceptor.setProperties` 用 `PropertyMapper.newInstance(properties)` 按 `@` 前缀抽出"插件类 + 配置子属性"，再反射新建 `InnerInterceptor` 加入列表：

```java
public void setProperties(Properties properties) {
    PropertyMapper pm = PropertyMapper.newInstance(properties);
    Map<String, Properties> group = pm.group(StringPool.AT);
    group.forEach((k, v) -> {
        InnerInterceptor innerInterceptor = ClassUtils.newInstance(k);
        innerInterceptor.setProperties(v);
        addInnerInterceptor(innerInterceptor);
    });
}
```

---

## 8. 核心 InnerInterceptor 详解

### 8.1 `OptimisticLockerInnerInterceptor`（318 行）

**改 `beforeUpdate`**：当 SQL 是 UPDATE 且参数包含 entity（`Constants.ENTITY`）→ 注入 version 字段 + 改写 SQL：

```java
public void beforeUpdate(Executor executor, MappedStatement ms, Object parameter) {
    if (SqlCommandType.UPDATE != ms.getSqlCommandType()) return;
    if (parameter instanceof Map) {
        Map<String, Object> map = (Map<String, Object>) parameter;
        doOptimisticLocker(map, ms.getId());
    }
}
```

`doOptimisticLocker`：
1. 取 entity（来自 `Constants.ENTITY`）
2. 找 `TableFieldInfo` 的 version 字段
3. 拿原 version 值 → 计算新 version 值（`VersionFactory.VERSION_FUNCTION_MAP` 支持 8 种 Java 类型）
4. 把 entity.set(version, newVal) 设置到实体上
5. 在 wrapper 上加 `apply(versionColumn + " = {0}", originalVersionVal)` 等价于 `where version = 1` 条件

**支持类型**：`long / Long / int / Integer / Date / Timestamp / LocalDateTime / Instant`。

### 8.2 `PaginationInnerInterceptor`（478 行，jsqlparser-4.9 副本）

**最大也最复杂的内置拦截器**——用 jsqlparser 解析 SQL 后注入 LIMIT/OFFSET，**对不同数据库用不同 dialect**。

**步骤概览**（节选自方法注释）：

```java
default void beforeQuery(...) {
    // 1. 是否需要分页
    IPage<?> page = (IPage<?>) ParameterUtils.findPage(parameter).orElse(null);
    if (page == null || page.getSize() < 0) return;
    
    // 2. 获取 dialect：MySqlDialect / OracleDialect / PostgreDialect / ...
    IDialect dialect = findDialect(configuration);
    
    // 3. 改写 SQL（Dialect.dialectPagination(...))
    String newSql = dialect.dialectPagination(originalSql, page);
    
    // 4. 重写 BoundSql
    new BoundSql(...);
}
```

内置 13 个 dialect：`MySqlDialect`、`PostgreDialect`、`OracleDialect`、`Oracle12cDialect`、`Db2Dialect`、`H2Dialect`、`Hive2Dialect`、`InformixDialect`、`SqlServer`、`SqlServer2005`、`SybaseDialect`、`TrinoDialect`、`GaussDBDialect`、`Gbase8sDialect`、`XCloudDialect`。

**`IDialect.dialectPagination(originalSql, page)`**：把 `SELECT * FROM user` 改写为：

| Dialect | 输出 |
|---|---|
| MySql | `SELECT * FROM user LIMIT ?, ?` |
| Postgre | `SELECT * FROM user LIMIT ? OFFSET ?` |
| Oracle | 使用 ROWNUM 嵌套 |
| SqlServer | `TOP n` 或 OFFSET/FETCH |

### 8.3 多租户 / 防全表更新 / 数据权限（不在内置，开源版第三方实现）

`com.baomidou.mybatisplus.extension.plugins.inner` 包下还提供：

- `ReplacePlaceholderInnerInterceptor`：占位符替换（如分页/动态表名）
- `IllegalSQLInnerInterceptor`（官方推荐使用 `IllegalSQLInterceptor` 自实现）：SQL 性能规范

社区有：`TenantLineInnerInterceptor`、`DataPermissionInterceptor`、`OptimisticLockerInnerInterceptor`、防全表更新的 `BlockAttackInnerInterceptor` 等都作为单独包提供（不在主仓库 main）。

---

## 9. `TableInfo`：反射元数据

### 9.1 加载时机

`TableInfoHelper.initTableInfo(...)` 在 `Configuration.addMapper(BaseMapper.class)` 时扫一次（`AbstractSqlInjector.inspectInject(...)` 内部），把 `Class<T>` → `TableInfo` 缓存。

### 9.2 字段

```java
public class TableInfo {
    private String tableName;
    private Class<?> entityType;
    private String keyProperty;              // 主键字段名
    private String keyColumn;                // 主键列名
    private IdType idType;
    private List<TableFieldInfo> fieldInfos; // 含逻辑删除、版本、字段填充、字段策略
    private TableFieldInfo logicDeleteFieldInfo;
    private TableFieldInfo versionFieldInfo;
    private boolean withLogicDelete;
    private boolean withVersion;
    private boolean withInsertFill;
    private boolean withUpdateFill;
    ...
}
```

`TableFieldInfo` 含：
- `column` 数据库列名
- `property` Java 字段名
- `fieldFill` 自动填充类型（INSERT / UPDATE / INSERT_UPDATE）
- `condition` SQL 拼接策略（NOT_NULL / NOT_EMPTY / ALWAYS）
- `logicDeleteValue` / `logicNotDeleteValue`

---

## 10. 数据流 ASCII 流程图

```
            ┌─── user code ─────────────────┐
            │ @Autowired UserMapper mapper;  │
            │ mapper.selectList(queryWrapper)│
            └────────────────┬───────────────┘
                             │ (1) Mapper proxy.invoke
                             ▼
            ┌─── DefaultSqlSession.selectList ───┐
            │    find MappedStatement.id         │
            │    call Executor.query              │
            └────────────────┬───────────────────┘
                             ▼
            ┌─── Executor (intercepted) ─────────┐
            │  Plugin.wrap(MybatisPlusInterceptor)│
            │  → intercept(Invocation)           │
            │    ├─ willDoQuery for each plugin  │
            │    ├─ beforeQuery for each plugin  │
            │    └─ executor.query() again  ←关键 │ ← 触达 CachingExecutor + L2 缓存
            └────────────────┬───────────────────┘
                             ▼
            ┌─── CachingExecutor ────────────────┐
            │  L2 缓存: ms.getCache()             │
            └────────────────┬───────────────────┘
                             ▼
            ┌─── BaseExecutor ─────────────────┐
            │  L1: PerpetualCache                │
            │  queryStack ++ for 嵌套查询        │
            └────────────────┬───────────────────┘
                             ▼
            ┌─── StatementHandler.prepare ─────┐
            │ (intercepted: beforePrepare)       │
            │ 设置 parameters / 解析 SQL        │
            └────────────────┬───────────────────┘
                             ▼
            ┌─── JDBC Statement.execute ──────┐
            └────────────────┬───────────────────┘
                             ▼
            ┌─── ResultSetHandler.handleRS ──┐
            └────────────────┬───────────────────┘
                             ▼
                       List<E> return
```

### 10.1 INSERT 流程（简化）

```
INSERT ─→ Cache NULL    ─→ Executor.update ─┐
                                          ▼ 拦截器链
                                  MybatisPlusInterceptor.intercept
                                       ├─ willDoUpdate → 块全表更新 (BlockAttackInnerInterceptor)
                                       ├─ beforeUpdate → OptimisticLockerInnerInterceptor 注 version
                                       ▼
                                  CachingExecutor.update  → MyBatis 执行
```

---

## 11. 与 MyBatis 3 / rbatis / rbatis-cache 对照速记表

| 维度 | MyBatis-Plus | MyBatis 3 | rbatis | rbatis-cache |
|---|---|---|---|---|
| 基底 | MyBatis 反射 | 自身 | 过程宏 | SPI 多 backend |
| 服务 | `IService<T>` + `ServiceImpl<M, T>` | 无 | `crud!()` 宏 | 不涉及 |
| Mapper | `BaseMapper<T>` 30+ 方法 | 手写 @Select | 宏展开（编译期） | 不涉及 |
| 条件构造 | `LambdaQueryWrapper<T>` | 不提供 | `value!{}` map | 不涉及 |
| 拦截器 SPI | `InnerInterceptor`（6 个钩子） | `Interceptor`（仅 1 个 intercept(Invocation)） | `Intercept`（before/after/ctx） | `get_or_load` 主体 |
| 拦截器 wrap | `MybatisPlusInterceptor` 一对多 | `Plugin.wrap` 一对一 | trait 链 push | 无 |
| 乐观锁 | `OptimisticLockerInnerInterceptor` | 不提供 | 不提供 | 不涉及 |
| 分页 | `PaginationInnerInterceptor` + 13 dialect | MyBatis RowBounds（offline） | `Intercept::ctx` + 自实现 | 不涉及 |
| 注解元数据 | 16 个 `@Table*`/`@Version`/`@IdType` | MyBatis 自带 @Select 等 | 无对应 | 无 |
| 缓存 | MyBatis-Plus 不提供新缓存，使用 MyBatis 自带 L2 | `Cache` SPI + 11 装饰器 | 内置 `CacheStore` + `MemoryCacheStore` | 多 backend (`LocalBackend`/`RedisCache`/...) |
| 多租户 | 第三方 InnerInterceptor | 不提供 | 不提供 | 不涉及 |
| 自动填充 | `MetaObjectHandler` | 不提供 | 不提供 | 不涉及 |

---

## 12. 关键设计权衡（FAQ）

### Q1：为什么 MyBatis-Plus 不直接基于 MyBatis 扩展，反而做了 `MybatisPlusInterceptor` "二次拦截器"？

`MybatisPlusInterceptor` 是一个 MyBatis `Interceptor`，里面又包了一组 `InnerInterceptor`。这样设计的两个原因：
- 集中调度：5 个拦截钩子由同一个拦截器包统一处理，避免每个 InnerInterceptor 都 wrap 一遍 Executor → 性能浪费
- 配置注入友好：使用 `@page / page:limit` 这种自描述协议，让 Spring Boot 配置可声明式加插件

### Q2：为什么 `PaginationInnerInterceptor` 自己调一次 `executor.query(...)`？

原始被拦截的 `Invocation.proceed()` 仍然会跑原 Executor 流程；**MyBatis-Plus 自己再调一次 `executor.query(...)`**，目的是：
- 这次调用进入 `CachingExecutor.query()` 才能让生成的 boundSql 走 L2 cache
- 否则一个长链 SQL"分页后才进缓存"，命中率会断崖式下降

源码注释里也写了"几乎不可能走进这里面,除非使用Executor的代理对象调用query[args[6]]"——这是个边角处理。

### Q3：`TableInfo` 是怎样被"反射元数据"共享的？

`TableInfoHelper.initTableInfo(...)` 在 `MapperRegistry.addMapper(...)` 时调一次，把 `Class<T>` → `TableInfo` 缓存到 `Map<String, TableInfo>`。**整张索引加载启动慢、运行期快**。注意：自定义类型 Generic 时一定要 `@TableName("xxx")` 准确指明表名，否则按类名（snake_case）兜底——经常踩到的坑。

### Q4：为什么 `LambdaQueryWrapper` 用 `User::getStatus` 而不是字符串 "status"？

通过 `SFunction<T, ?>` 拿到 lambda 表达式的 attribute（`User::getStatus`），再通过反射得到 `Method#getName` = "getStatus" → "status" —— 这就是 Java ASM 上 `SerializedLambda` 提取：编译期 lambda 序列化为 `SerializedLambda`，运行期 reflect 出 method name。**这是 MyBatis-Plus v3 起的稳定方案**。注意：项目里 lombok 或 @Accessors(chain=true) 会破坏 lambda 提取，需要额外配置或用 `getter replacement`。

### Q5：`OptimisticLockerInnerInterceptor` 在 wrapper 模式下为什么需要字段名+状态机扫描？

当 `update(T, Wrapper<T>)` 调用时，wrapper 里可能已包含 `WHERE version = 1` 条件——状态机 `INIT → FIELD_FOUND → EQ_FOUND → VERSION_VALUE_PRESENT` 用来：
1. 找到 wrapper 中"version 字段"
2. 确认存在 EQ 关键字
3. 确认右侧有 `#{}` 占位
4. 在 `paramNameValuePairs` 里更新 value = newVal + 把 `version = #{oldVal}` 改写为 `version = #{newVal}`

**避免了"where version=1 and set version=2 但更新条件自己又写 where version=1" 的循环**。

### Q6：`MyBatis-Plus` SQL 注入器为什么做成"每个方法一个文件 + 各方法继承 AbstractMethod + 一个 DefaultSqlInjector 集中组织"？

让用户能：
1. 替换"我自己的 `DefaultSqlInjector`"只保留部分方法（性能敏感场景）
2. 新增自定义 `AbstractMethod` 子类继承并注入用户 XML mapper 的 SQL 文本

### Q7：`@InterceptorIgnore` 注解是怎么和 `MybatisPlusInterceptor` 协作的？

`MybatisPlusInterceptor.intercept(...)` 内：

```java
if (InterceptorIgnoreHelper.willIgnoreCurrent(...)) {
    return invocation.proceed();   // 跳过全部 plugin
}
```

`InterceptorIgnoreHelper` 通过注解 + intercept signature 维护一个"忽略集合"，**让某些 SQL 不被拦截**（例如：分页插件里查 count 时不能被分页）

### Q8：与 rbatis 的类比 → 如果要给 rbatis 写 `BaseMapper<T>` 风格 API：

rbatis 已经有了 `crud!{}` 宏——比 MyBatis-Plus 在编译期展开的版本更静态。如果要实现"接口继承"风格：可以仿 rbatis 现在的 `intercept::ctx` 增加一个"用 trait 模拟 BaseMapper 接口绑定"——这是另一项工作，本文档不展开。

---

## 13. codegraph 速查命令

```bash
export PATH="/Users/wandl/.nvm/versions/node/v24.18.0/bin:$PATH"
cd /Users/wandl/workspaces/workspace-github/mybatis-plus

codegraph status                              # 1056 文件 / ≈ 14k 符号
codegraph query "BaseMapper\|IService\|ServiceImpl"
codegraph query "DefaultSqlInjector\|AbstractMethod\|AbstractSqlInjector"
codegraph query "InnerInterceptor\|MybatisPlusInterceptor\|@Intercepts"
codegraph query "OptimisticLockerInnerInterceptor\|PaginationInnerInterceptor"
codegraph query "AbstractWrapper\|LambdaQueryWrapper\|LambdaUpdateWrapper"
codegraph query "TableInfo\|TableFieldInfo\|TableInfoHelper"
codegraph query "SqlMethod\|SqlKeyword\|SqlInjectionUtils"
codegraph query "Wrappers\|allEq\|apply"
codegraph query "MetaObjectHandler"
codegraph query "Repository\|AbstractRepository"
codegraph query "SqlInjector\|KeyGenerator\|GenericTypeAware"   # SPI 包
```

---

## 14. 推荐阅读顺序

1. `mybatis-plus-annotation/` 所有 `@Table*` 注解（16 个）
2. `mybatis-plus-core/mapper/BaseMapper.java` —— 624 行的"主入口"
3. `mybatis-plus-core/injector/DefaultSqlInjector.java` —— 14 个方法注册表
4. `mybatis-plus-core/injector/methods/{Insert,UpdateById,SelectList}.java` —— 单方法样本
5. `mybatis-plus-core/enums/SqlMethod.java` —— 14 个 SQL 模板字符串
6. `mybatis-plus-core/conditions/{Wrapper,AbstractWrapper}.java` —— 条件构造器
7. `mybatis-plus-core/metadata/{TableInfo,TableFieldInfo,TableInfoHelper}.java` —— 元数据
8. `mybatis-plus-extension/plugins/inner/InnerInterceptor.java` —— SPI
9. `mybatis-plus-extension/plugins/MybatisPlusInterceptor.java` —— 156 行"分发器"
10. `mybatis-plus-extension/plugins/inner/OptimisticLockerInnerInterceptor.java` —— 完整理智模型
11. `mybatis-plus-jsqlparser-support/.../PaginationInnerInterceptor.java`（478 行）—— 最复杂的 InnerInterceptor
12. `mybatis-plus-spring/service/{IService,ServiceImpl}.java` —— 业务 Service 层
13. `mybatis-plus-generator/` —— 代码生成器（自动搭出 mapper / service / entity / controller）

---

## 15. 未变 / 已废弃 / 注意点

1. **`@Deprecated` 方法**：`BaseMapper.deleteBatchIds`/`selectBatchIds` 等使用 `@Deprecated` 标注，请用 `deleteByIds`/`selectByIds`。删除版本从 `3.5.7` 起。
2. **`MybatisSqlSessionFactoryBean`**：在 `mybatis-plus-extension` 包的 `MybatisSqlSessionFactoryBean.java` 之外，**Spring Boot Starter** 里还有独立的 `MybatisPlusInterceptorAutoConfiguration`（依靠 `META-INF/spring.factories`）。
3. **`@TableName`** 没指定表名会按类名 snake_case 兜底：当 `User` 类型 → `t_user`（v3.5+ 默认前缀）。
4. **逻辑删除字段**：`@TableLogic` 标记后，`SELECT * FROM user` 自动加 `WHERE deleted=0`；`DELETE FROM user` 转 `UPDATE ... SET deleted=1`。
5. **乐观锁**：`@Version` 必须写在 `int/Integer/long/Long/Date/Timestamp/LocalDateTime/Instant` 八种类型之一——其他类型会被原样返回（**不会自动抛错**，需要自己改造）。
6. **`Wrapper.apply(String sqlSegment, Object... params)`**：原生 SQL 注入点；如果 `sqlSegment` 拼接用户输入 → SQL 注入风险（**勿用于 untrusted input**）。
7. **`@InterceptorIgnore`**：参数解析看 `MybatisPlusInterceptor.intercept(...)` 内的"忽略列表"机制。它与 `@InterceptorIgnore(tenantLine = "true")` 字段级注解共同支持跳过某些拦截器。
8. **生成器 (Generator)**：老的 `AutoGenerator` 入口路径仍在 `mybatis-plus-generator`，但已废弃。新版叫 `MybatisPlusGenerator`（`MyBatis 3.5+`）。**当前版本号 3.5.17 仍是 v3.x 系列**，v4 还在 Roadmap。
9. **多租户**：`TenantLineInnerInterceptor` 等在主仓库**不在 `main` 中**——作为独立模块发布，需用户自行引用 Maven 工件。
10. **调试模式**：`<setting name="logImpl" value="StdOutImpl"/>` 输出 SQL，可配合 `p6spy` 做。MyBatis-Plus 不强制。

---

## 附录 A：缓存 / 事务的传播点

虽然 MyBatis-Plus 不专门做缓存，但 MyBatis-Plus 的设计对缓存非常友好：

| 关注点 | MyBatis-Plus 行为 |
|---|---|
| `@CacheNamespace` | MyBatis 自带；MyBatis-Plus 通过 `@TableName` + `@Version` 不变更 cache key |
| L2 cache 默认值 | 通过 MyBatis `<cache/>` 显式声明 |
| 拦截器 vs 缓存 | MyBatis-Plus 拦截器在 `Executor.query` 之前，`CachingExecutor` 之外 —— 缓存命中时不再走拦截器（性能优化） |
| Multi-tenant 缓存隔离 | `tenantLine` 拼接进 SQL → CachingExecutor cache key 自动隔离 |

---

## 附录 B：与 rbatis-plus 的协作（本仓库 `rbatis-plus`）

`/Users/wandl/workspaces/workspace-github-easy-4-rust/rbatis-plus` 是把本仓库设计思路（BaseMapper / Wrapper / InnerInterceptor）移植到 Rust 的实现。本仓库文档对它的设计契约：

| 维度 | MyBatis-Plus | rbatis-plus（Rust 移植） |
|---|---|---|
| `BaseMapper<T>` | Java 接口继承 | Rust trait + 宏 |
| `IService<T>` | Spring `@Autowired` | 各用 Rust DI / 裸函数 |
| `Wrapper<T>` | `LambdaQueryWrapper` | 暂无 plan（可能用 builder-pattern 链式） |
| `InnerInterceptor` | Java SPI + JDK 代理 | `Intercept` trait（已有，主仓 `df87ac41`） |
| SqlInjector | `DefaultSqlInjector + AbstractMethod` | 编译期过程宏（主仓已用此方案） |
| 分页 | 13 dialect → jsqlparser | `Intercept::ctx` 改写 SQL 即可 |
| 乐观锁 | Interceptor inspect 参数 map | 类似实现（待做） |

——这个对照可直接当 rbatis-plus 后续工作的 checklist。
