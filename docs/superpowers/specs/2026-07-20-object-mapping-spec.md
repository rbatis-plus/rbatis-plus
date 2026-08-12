# mybatis-plus → rbatis-plus 对象级对照表

- **日期**：2026-07-20
- **状态**：已实施（部分进行中）
- **基线**：baomidou/mybatis-plus v3.5.17 + mybatis-plus-enhance 2.0.x + mybatis-3 3.6.0 + rbatis-wrapper 0.1.1

---

## 1. 目标工程结构

```text
rbatis-plus/                                              Rust 多模块工程
├── Cargo.toml                                         workspace 根
├── rbatis-plus-core/                              核心引擎
│   └── src/
│       ├── cache/                                 复用 rbatis::plugin::cache
│       ├── conditions/                            条件构造器
│       ├── derive/                                16 derive trait 定义
│       ├── mapper/                                BaseMapper trait
│       ├── metadata/                              TableInfo / TableFieldInfo
│       ├── method/                                AbstractMethod 子类
│       └── page.rs                                Page<T> + PageRequest
├── rbatis-plus-macros/                            过程宏
│   └── src/lib.rs                                 proc-macro 入口
├── rbatis-plus-extension/                         扩展（InnerInterceptor + 增强能力）
│   └── src/
│       ├── crypto/                                加解密
│       ├── i18n/                                  国际化
│       ├── inner/                                 14 InnerInterceptor
│       ├── insert_ignore/                         INSERT IGNORE
│       ├── observation/                           SQL 观测
│       ├── service/                               IService / ServiceImpl
│       └── signature/                             签名/验签
├── rbatis-plus-sqlparser/                         SQL 解析 + 方言
├── rbatis-plus-vernal/                            Axum/Actix 集成
├── rbatis-plus-generator/                         代码生成器
└── src/lib.rs                                     facade crate
```

---

## 2. 状态图例

| 标记 | 含义 |
|---|---|
| ✅ | 已迁移，独立文件已对齐 |
| 🔀 | 语义已迁移但与其他对象合并在同一文件，待拆分 |
| ⬜ | 未迁移，缺失 |
| 🔶 | 语义等价但形态不同 |
| 🚫 | 不迁移（Java 生态特有） |

---

## 3. annotation（16） — mybatis-plus-annotation

| Java 类 | 目标文件 (rbatis-plus-macros/src/) | 状态 | 说明 |
|---|---|---|---|
| `@TableName` | `derive/table_name.rs` | ✅ | 已落 |
| `@TableId` | `derive/table_id.rs` | ✅ | 已落 |
| `@TableField` | `derive/table_field.rs` | ✅ | 已落 |
| `@TableLogic` | `derive/table_logic.rs` | ✅ | 已落 |
| `@Version` | `derive/version.rs` | ✅ | 已落 |
| `@EnumValue` | `derive/enum_value.rs` | ⬜ | 待建 |
| `@FieldFill` | `derive/field_fill.rs` | ✅ | 已落 |
| `@FieldStrategy` | `derive/field_strategy.rs` | ✅ | 已落 |
| `@InterceptorIgnore` | `derive/interceptor_ignore.rs` | ✅ | 已落 |
| `@KeySequence` | `derive/key_sequence.rs` | ✅ | 已落 |
| `@OrderBy` | `derive/order_by.rs` | ✅ | 已落 |
| `@SqlCondition` | `derive/sql_condition.rs` | ⬜ | 待建 |
| `DbType` | `db_type.rs` | ✅ | 已落 |
| `IEnum` | `i_enum.rs` | ✅ | 已落 |
| `IdType` | `id_type.rs` | ✅ | 已落 |
| `IgnoreEncrypted` | `encrypted_field.rs` | ✅ | 已落 |

---

## 4. conditions（28） — mybatis-plus-core

| Java 类 | 目标文件 (rbatis-plus-core/src/conditions/) | 状态 | 说明 |
|---|---|---|---|
| `Wrapper<T>` | `wrapper.rs` | ✅ | 29 行 |
| `AbstractWrapper<T,R,Children>` | `abstract_wrapper.rs` | ✅ | 202 行 |
| `AbstractLambdaWrapper<T,Children>` | `abstract_lambda_wrapper.rs` | ⬜ | 待建 |
| `QueryWrapper<T>` | `query/query_wrapper.rs` | ✅ | 317 行 |
| `UpdateWrapper<T>` | `update/update_wrapper.rs` | ✅ | 189 行 |
| `LambdaQueryWrapper<T>` | `query/lambda_query_wrapper.rs` | ✅ | 548 行 |
| `LambdaUpdateWrapper<T>` | `update/lambda_update_wrapper.rs` | ✅ | 534 行 |
| `Compare<R>` | `compare.rs` | ✅ | 209 行 |
| `Func<R>` | `func.rs` | ✅ | 119 行 |
| `Nested<R>` | `nested.rs` | ✅ | 137 行 |
| `Join<R>` | `join.rs` | ✅ | 已落 |
| `MergeSegments` | `merge_segments.rs` | ✅ | 113 行 |
| `Column<F>` | `query/column.rs` | ✅ | 72 行 |
| `SharedString` | `shared_string.rs` | ⬜ | 待建 |
| `ISqlSegment` | `i_sql_segment.rs` | ⬜ | 待建 |
| `NormalSegmentList` | `normal_segment_list.rs` | ⬜ | 待建 |
| `GroupBySegmentList` | `group_by_segment_list.rs` | ⬜ | 待建 |
| `OrderBySegmentList` | `order_by_segment_list.rs` | ⬜ | 待建 |
| `HavingSegmentList` | `having_segment_list.rs` | ⬜ | 待建 |
| `ColumnSegment` | `column_segment.rs` | ⬜ | 待建 |
| `Wrappers` | `wrappers.rs` | ⬜ | 待建 |
| `SqlKeyword` | (enums) `sql_keyword.rs` | ⬜ | 待建 |
| `SqlLike` | (enums) `sql_like.rs` | ⬜ | 待建 |
| `WrapperKeyword` | (enums) `wrapper_keyword.rs` | ⬜ | 待建 |
| `Constants` | `constants.rs` | ⬜ | 待建 |

---

## 5. mapper（3） — mybatis-plus-core

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `BaseMapper<T>` | `mapper/base_mapper.rs` | ✅ | 90+ 行 trait |
| `Mapper<T>` | `mapper/mapper.rs` | ⬜ | 待建 |
| `MapperProxyMetadata` | `mapper/mapper_proxy_metadata.rs` | ⬜ | 待建 |

---

## 6. metadata（8） — mybatis-plus-core

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `TableInfo` | `metadata/table_info.rs` | ✅ | 142 行 |
| `TableFieldInfo` | `metadata/table_field_info.rs` | 🔀 | 合并在 table_info.rs |
| `TableIdInfo` | `metadata/table_id_info.rs` | ⬜ | 待建 |
| `OrderFieldInfo` | `metadata/order_field_info.rs` | ⬜ | 待建 |
| `MetaObject` | `metadata/meta_object.rs` | ⬜ | 待建 |
| `ColumnCache` | `metadata/column_cache.rs` | ⬜ | 待建 |
| `TableInfoHelper` | `metadata/table_info_helper.rs` | ⬜ | 待建 |
| `TableInfoHelperFactory` | `metadata/table_info_helper_factory.rs` | ⬜ | 待建 |

---

## 7. method（21） — mybatis-plus-core

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AbstractMethod` | `method/abstract_method.rs` | ✅ | 已落 |
| `SqlMethod` | `method/sql_method.rs` | ✅ | 已落 |
| `Insert` | `method/insert.rs` | ✅ | 已落 |
| `Delete` | `method/delete.rs` | ✅ | 已落 |
| `DeleteById` | `method/delete_by_id.rs` | ✅ | 已落 |
| `DeleteByMap` | `method/delete_by_map.rs` | ⬜ | 待建 |
| `DeleteByIds` | `method/delete_by_ids.rs` | ✅ | 已落 |
| `DeleteBatchByIds` | `method/delete_batch_by_ids.rs` | ⬜ | 待建 |
| `Update` | `method/update.rs` | ✅ | 已落 |
| `UpdateById` | `method/update_by_id.rs` | ✅ | 已落 |
| `SelectById` | `method/select_by_id.rs` | ✅ | 已落 |
| `SelectByIds` | `method/select_by_ids.rs` | ✅ | 已落 |
| `SelectBatchByIds` | `method/select_batch_by_ids.rs` | ⬜ | 待建 |
| `SelectByMap` | `method/select_by_map.rs` | ✅ | 已落 |
| `SelectCount` | `method/select_count.rs` | ✅ | 已落 |
| `SelectList` | `method/select_list.rs` | ✅ | 已落 |
| `SelectMaps` | `method/select_maps.rs` | ✅ | 已落 |
| `SelectMapsPage` | `method/select_maps_page.rs` | ⬜ | 待建 |
| `SelectObjs` | `method/select_objs.rs` | ✅ | 已落 |
| `SelectOne` | `method/select_one.rs` | ✅ | 已落 |
| `SelectPage` | `method/select_page.rs` | ⬜ | 待建 |
| `SelectWithCursor` | `method/select_with_cursor.rs` | ⬜ | 待建 |

---

## 8. extension inner（14） — mybatis-plus-extension + enhance

### 8.1 mybatis-plus 原生 inner

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `InnerInterceptor` | `inner/inner_interceptor.rs` | ✅ | 76 行 |
| `PaginationInnerInterceptor` | `inner/pagination.rs` | ✅ | 217 行 |
| `TenantLineInnerInterceptor` | `inner/tenant.rs` | ✅ | 96 行 |
| `DataPermissionInnerInterceptor` | `inner/data_permission.rs` | ✅ | 80 行 |
| `BlockAttackInnerInterceptor` | `inner/block_attack.rs` | ✅ | 50 行 |
| `DynamicTableNameInnerInterceptor` | `inner/dynamic_table_name.rs` | ✅ | 71 行 |
| `OptimisticLockerInnerInterceptor` | `inner/optimistic_locker.rs` | ✅ | 68 行 |
| `IllegalSQLInnerInterceptor` | `inner/illegal_sql.rs` | ⬜ | 待建 |
| `DataChangeRecorderInnerInterceptor` | `inner/data_change_recorder.rs` | ⬜ | 待建 |
| `ReplacePlaceholderInnerInterceptor` | `inner/replace_placeholder.rs` | ⬜ | 待建 |

### 8.2 mybatis-plus-enhance inner

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `EnhanceInnerInterceptor` | `inner/enhance_interceptor.rs` | ✅ | 已落 |
| `EnhancePhase` | `inner/enhance_phase.rs` | ✅ | 已落 |
| `MybatisPlusEnhanceInterceptor` | `inner/mybatis_plus_enhance_interceptor.rs` | ✅ | 已落 |
| `DataEncryptionInnerInterceptor` | `crypto/data_encryption.rs` | ✅ | MVP |
| `DataDecryptionInnerInterceptor` | `crypto/data_decryption.rs` | ✅ | 已落 |
| `DataSignatureInnerInterceptor` | `signature/data_signature.rs` | ✅ | MVP |
| `DataI18nInnerInterceptor` | `i18n/data_i18n.rs` | ✅ | 已落 |
| `LongSqlInnerInterceptor` | `inner/long_sql.rs` | ✅ | 已落 |
| `SqlObservationInnerInterceptor` | `observation/sql_observation.rs` | ✅ | 已落 |

---

## 9. extension service — mybatis-plus-extension

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `IService<T>` | `service/i_service.rs` | ✅ | 85 行 |
| `ServiceImpl<M,T>` | `service/service_impl.rs` | ✅ | 337 行 |

---

## 10. generator — mybatis-plus-generator

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| `AutoGenerator` | `auto_generator.rs` | ✅ | 已落 |
| `DataSourceConfig` | `config/data_source.rs` | ✅ | 108 行 |
| `PackageConfig` | `config/package.rs` | ✅ | 115 行 |
| `StrategyConfig` | `config/strategy.rs` | ✅ | 156 行 |
| `GlobalConfig` | `config/global.rs` | ✅ | 67 行 |
| `TemplateEngine` | `template/template_engine.rs` | ✅ | 已落 |
| `TeraTemplateEngine` | `template/tera_engine.rs` | ✅ | 268 行 |
| `HandlebarsTemplateEngine` | `template/handlebars_engine.rs` | ✅ | 已落 |
| `AskamaTemplateEngine` | `template/askama_engine.rs` | ✅ | 已落 |
| `MaudTemplateEngine` | `template/maud_engine.rs` | ✅ | 已落 |
| `TableInfo` query | `query/table_info.rs` | ✅ | 63 行 |

---

## 11. vernal — mybatis-plus-spring

| Java 类 | 目标文件 | 状态 | 说明 |
|---|---|---|---|
| Axum 集成 | `axum_integration.rs` | ✅ | 已落 |
| Actix 集成 | `actix_integration.rs` | ✅ | 已落 |
| SqlRunner | `sql_runner.rs` | ✅ | 已落 |
| AppState | `state.rs` | ✅ | 已落 |
| TransactionTemplate | `transaction.rs` | ✅ | 已落 |

---

## 12. 初始盘点统计

| 状态 | 数量 | 占比 |
|---|---|---|
| ✅ 已对齐 | ~80 | ~50% |
| 🔀 合并未拆 | ~10 | ~6% |
| ⬜ 待迁移 | ~65 | ~40% |
| 🔶 形态不同 | ~5 | ~3% |
| 🚫 不迁移 | ~2 | ~1% |
