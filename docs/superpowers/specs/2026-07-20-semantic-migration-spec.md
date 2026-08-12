# mybatis-plus → rbatis-plus 功能语义迁移对照表

- **日期**：2026-07-20
- **状态**：已实施（部分进行中）
- **基线**：baomidou/mybatis-plus v3.5.17 + mybatis-plus-enhance 2.0.x + mybatis-3 3.6.0 + rbatis-wrapper 0.1.1
- **迁移原则**：功能语义对齐，实现方式 Rust 化

---

## 状态图例

| 标记 | 含义 |
|---|---|
| ✅ | 已迁移并有测试 |
| 🔶 | 语义等价但形态不同 |
| ⬜ | 未迁移（路线图） |

---

## 1. 基础 CRUD（BaseMapper）

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| `BaseMapper<T>` 接口 | `BaseMapper<T>` trait | ✅ | trait 已声明 |
| `insert(T entity)` | `async fn insert(&self, entity: &T) -> Result<u64>` | 🔶 | iter3 加默认 impl |
| `deleteById(Serializable id)` | `async fn delete_by_id(&self, id: &Value) -> Result<u64>` | 🔶 | iter3 |
| `deleteByMap(Map)` | `async fn delete_by_map(&self, column_map: Map) -> Result<u64>` | ⬜ | iter3 |
| `delete(Wrapper<T>)` | `async fn delete(&self, wrapper: &QueryWrapper, table: &str) -> Result<u64>` | 🔶 | iter3 |
| `deleteByIds(Collection)` | `async fn delete_by_ids(&self, ids: Collection) -> Result<u64>` | 🔶 | iter3 |
| `updateById(T entity)` | `async fn update_by_id(&self, entity: &T) -> Result<u64>` | 🔶 | iter3 |
| `update(T, Wrapper<T>)` | `async fn update(&self, entity: &T, wrapper: &UpdateWrapper, table: &str)` | 🔶 | iter3 |
| `selectById(Serializable id)` | `async fn select_by_id(&self, id: &Value) -> Result<Option<T>>` | 🔶 | iter3 |
| `selectByIds(Collection)` | `async fn select_by_ids(&self, ids: Collection) -> Result<Vec<T>>` | 🔶 | iter3 |
| `selectByMap(Map)` | `async fn select_by_map(&self, column_map: Map) -> Result<Vec<T>>` | 🔶 | iter3 |
| `selectOne(Wrapper<T>)` | `async fn select_one(&self, wrapper: &QueryWrapper, table: &str) -> Result<Option<T>>` | 🔶 | iter3 |
| `selectCount(Wrapper<T>)` | `async fn select_count(&self, wrapper: &QueryWrapper, table: &str) -> Result<u64>` | 🔶 | iter3 |
| `selectList(Wrapper<T>)` | `async fn select_list(&self, wrapper: &QueryWrapper, table: &str) -> Result<Vec<T>>` | 🔶 | iter3 |
| `selectMaps(Wrapper<T>)` | `async fn select_maps(...)` | 🔶 | iter3 |
| `selectObjs(Wrapper<T>)` | `async fn select_objs(...)` | 🔶 | iter3 |
| `selectPage(P, Wrapper<T>)` | `async fn select_page(&self, wrapper, table, page_no, page_size) -> Result<Page<T>>` | ⬜ | iter3 |
| `selectMapsPage(P, Wrapper<T>)` | `async fn select_maps_page(...)` | ⬜ | iter3 |
| `insertOrUpdate(T entity)` | `async fn insert_or_update(&self, entity: &T) -> Result<bool>` | ⬜ | iter3 |

---

## 2. 条件构造器（Conditions）

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| `QueryWrapper<T>` | `QueryWrapper<T>` | ✅ | 317 行 |
| `UpdateWrapper<T>` | `UpdateWrapper<T>` | ✅ | 189 行 |
| `LambdaQueryWrapper<T>` | `LambdaQueryWrapper<T>` | ✅ | 548 行，类型安全列引用 |
| `LambdaUpdateWrapper<T>` | `LambdaUpdateWrapper<T>` | ✅ | 534 行 |
| `AbstractWrapper.eq/ne/gt/ge/lt/le` | `Compare` trait | ✅ | 209 行 |
| `AbstractWrapper.like/in/is_null` | `Func` trait | ✅ | 119 行 |
| `AbstractWrapper.or/and/nested` | `Nested` trait | ✅ | 137 行 |
| `AbstractWrapper.group_by/order_by/having` | `Func` trait | ✅ | 已落 |
| `SFunction<T,R>` lambda 列引用 | `Column<F>` + PhantomData | 🔶 | Rust 用泛型替代 Java 反射 |
| `Wrappers.query()/update()` 工厂 | `Wrappers` | ⬜ | 待建 |

---

## 3. 元数据（Metadata）

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| `TableInfo` | `TableInfo` | ✅ | 142 行 |
| `TableFieldInfo` | `TableFieldInfo` | 🔀 | 合并在 table_info.rs |
| `TableIdInfo` | `TableIdInfo` | ⬜ | 待建 |
| `OrderFieldInfo` | `OrderFieldInfo` | ⬜ | 待建 |
| `MetaObject` | `MetaObject` | ⬜ | 待建 |
| `ColumnCache` | `ColumnCache` | ⬜ | 待建 |
| `TableInfoHelper` | `TableInfoHelper` | ⬜ | 待建 |

---

## 4. 注解 → Derive 宏

| Java 注解 | Rust Derive 宏 | 状态 | 说明 |
|---|---|---|---|
| `@TableName` | `#[derive(TableName)]` | ✅ | 已落 |
| `@TableId` | `#[derive(TableId)]` | ✅ | 已落 |
| `@TableField` | `#[derive(TableField)]` | ✅ | 已落 |
| `@TableLogic` | `#[derive(TableLogic)]` | ✅ | 已落 |
| `@Version` | `#[derive(Version)]` | ✅ | 已落 |
| `@FieldFill` | `#[derive(FieldFill)]` | ✅ | 已落 |
| `@FieldStrategy` | `#[derive(FieldStrategy)]` | ✅ | 已落 |
| `@InterceptorIgnore` | `#[derive(InterceptorIgnore)]` | ✅ | 已落 |
| `@KeySequence` | `#[derive(KeySequence)]` | ✅ | 已落 |
| `@OrderBy` | `#[derive(OrderBy)]` | ✅ | 已落 |
| `@EnumValue` | `#[derive(EnumValue)]` | ⬜ | 待建 |
| `@SqlCondition` | `#[derive(SqlCondition)]` | ⬜ | 待建 |
| `@EncryptedField` | `#[derive(EncryptedField)]` | ✅ | enhance |
| `@EncryptedTable` | `#[derive(EncryptedTable)]` | ✅ | enhance |
| `@I18nColumn` | `#[derive(I18nColumn)]` | ✅ | enhance |
| `@SignatureField` | `#[derive(SignatureField)]` | ✅ | enhance |

---

## 5. 拦截器（InnerInterceptor）

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| `PaginationInnerInterceptor` | `PaginationInnerInterceptor` | ✅ | 217 行 |
| `TenantLineInnerInterceptor` | `TenantInnerInterceptor` | ✅ | 96 行 |
| `DataPermissionInnerInterceptor` | `DataPermissionInnerInterceptor` | ✅ | 80 行 |
| `BlockAttackInnerInterceptor` | `BlockAttackInnerInterceptor` | ✅ | 50 行 |
| `DynamicTableNameInnerInterceptor` | `DynamicTableNameInnerInterceptor` | ✅ | 71 行 |
| `OptimisticLockerInnerInterceptor` | `OptimisticLockerInnerInterceptor` | ✅ | 68 行 |
| `IllegalSQLInnerInterceptor` | `IllegalSQLInnerInterceptor` | ⬜ | 待建 |
| `DataChangeRecorderInnerInterceptor` | `DataChangeRecorderInnerInterceptor` | ⬜ | 待建 |
| `ReplacePlaceholderInnerInterceptor` | `ReplacePlaceholderInnerInterceptor` | ⬜ | 待建 |
| `DataEncryptionInnerInterceptor` | `DataEncryptionInnerInterceptor` | ✅ | MVP |
| `DataDecryptionInnerInterceptor` | `DataDecryptionInnerInterceptor` | ✅ | 已落 |
| `DataSignatureInnerInterceptor` | `DataSignatureInnerInterceptor` | ✅ | MVP |
| `DataI18nInnerInterceptor` | `DataI18nInnerInterceptor` | ✅ | 已落 |
| `LongSqlInnerInterceptor` | `LongSqlInnerInterceptor` | ✅ | 已落 |
| `SqlObservationInnerInterceptor` | `SqlObservationInnerInterceptor` | ✅ | 已落 |
| `InsertIgnoreInnerInterceptor` | `InsertIgnoreInnerInterceptor` | ✅ | MVP |

---

## 6. 服务层（Service）

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| `IService<T>` | `IService<T>` trait | ✅ | 85 行 |
| `ServiceImpl<M,T>` | `ServiceImpl<M,T>` | ✅ | 337 行 |
| `IRepository<T>` | `IRepository<T>` | ⬜ | 待建 |
| `AbstractRepository<T>` | `AbstractRepository<T>` | ⬜ | 待建 |

---

## 7. 代码生成器（Generator）

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| `AutoGenerator` | `AutoGenerator` | ✅ | 已落 |
| `DataSourceConfig` | `DataSourceConfig` | ✅ | 108 行 |
| `PackageConfig` | `PackageConfig` | ✅ | 115 行 |
| `StrategyConfig` | `StrategyConfig` | ✅ | 156 行 |
| `GlobalConfig` | `GlobalConfig` | ✅ | 67 行 |
| `TemplateEngine` | `TemplateEngine` trait | ✅ | 已落 |
| FreeMarker → Tera | `TeraTemplateEngine` | ✅ | 268 行 |
| Velocity → Handlebars | `HandlebarsTemplateEngine` | ✅ | 已落 |
| JSP/Thymeleaf → Askama | `AskamaTemplateEngine` | ✅ | 已落 |
| Twirl/JSX → maud | `MaudTemplateEngine` | ✅ | 已落 |

---

## 8. 集成层（Vernal / Spring）

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| Spring Boot Starter | rbatis-plus-vernal | 🔶 | Axum/Actix 替代 Spring |
| `@MapperScan` | auto_config | ⬜ | 待建 |
| `SqlSessionFactory` | `RBatis` | ✅ | 复用上游 |
| `@Transactional` | `TransactionalGuard` | ✅ | 已落 |
| `SqlRunner` | `SqlRunner` | ✅ | 已落 |

---

## 9. 缓存系统

| Java 语义 | Rust 等价实现 | 状态 | 说明 |
|---|---|---|---|
| MyBatis `Cache` SPI | `rbatis::plugin::cache::CacheStore` | 🚫 | 复用上游 |
| `CacheIntercept` | `CacheIntercept` | ⬜ | iter6 切换 |
| `MemoryCacheStore` | `MemoryCacheStore` (DashMap) | 🔶 | 待迁移到 moka |
| `RedisCacheStore` | `RedisCacheStore` | ⬜ | iter6 新建 |
| `TransactionalCache` | `TransactionalCacheBuffer` | 🚫 | 复用上游 |

---

## 10. Rust 化映射规则

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
