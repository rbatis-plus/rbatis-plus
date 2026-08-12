# rbatis-plus 对象名称一致性规范

- **日期**：2026-07-20
- **状态**：已实施
- **基线**：baomidou/mybatis-plus v3.5.17 + mybatis-plus-enhance 2.0.x + mybatis-3 3.6.0 + rbatis-wrapper 0.1.1

---

## 1. 命名规则总览

| 类别 | Java | Rust | 示例 |
|---|---|---|---|
| **目录名** | `.` 分隔包名 | snake_case `/` 分隔 | `com.baomidou.mybatisplus.core.conditions` → `conditions/` |
| **文件名** | `PascalCase.java` | `snake_case.rs` | `BaseMapper.java` → `base_mapper.rs` |
| **类型名** | `PascalCase` | `PascalCase`（不变） | `BaseMapper` → `BaseMapper` |
| **方法名** | `camelCase` | `snake_case` | `selectById` → `select_by_id` |
| **字段名** | `camelCase` | `snake_case` | `keyColumn` → `key_column` |
| **常量名** | `UPPER_SNAKE_CASE` | `UPPER_SNAKE_CASE`（不变） | `Constants.WRAPPER` → `Constants::WRAPPER` |
| **泛型参数** | 单大写字母 | 单大写字母（不变） | `<T, R, Children>` → `<T, R, Children>` |
| **枚举变体** | `UPPER_SNAKE_CASE` | `PascalCase`（Rust 惯例） | `IdType.NONE` → `IdType::None` |
| **trait 名** | interface `IXxx` | trait `Xxx`（去 I 前缀） | `ISqlInjector` → `SqlInjector` |
| **参数名** | `camelCase` | `snake_case` | `parameterObject` → `parameter_object` |

---

## 2. 特殊处理规则

### 2.1 Java interface `I` 前缀

`ISqlInjector` → trait 名 `SqlInjector`（Rust 惯例不带 I）；但文件名保留 `i_sql_injector.rs` 以维持"一文件一 Java 对象"映射。

### 2.2 Java 内部类

`TableInfo.EntityKey` → 文件 `table_info.rs` 内嵌（不拆出）；如必须拆则 `table_info_entity_key.rs`。

### 2.3 Rust 关键字冲突

`type` / `ref` / `match` / `mod` / `move` / `self` / `super` / `where` / `yield` / `async` / `await` / `dyn` / `fn` / `in` / `let` / `use` → 加 `_` 后缀（如 `r#type` 或 `type_`）。

### 2.4 方法名冲突

同一 trait impl 多个父 trait 时方法名冲突 → 用 fully-qualified 语法 `<<Self as Compare>::eq`。

### 2.5 Java `get/set` 前缀

Rust 不加 `get_` 前缀（用 field 直接访问）；但保留 `set_` 前缀因为 Rust 没有 setter 约定。

---

## 3. 核心对象 snake_case 转换表

### 3.1 annotation（16）

| Java 类 | 文件名 | Rust 类型 | 说明 |
|---|---|---|---|
| `TableName` | `table_name.rs` | `TableName`（derive） | |
| `TableId` | `table_id.rs` | `TableId`（derive） | |
| `TableField` | `table_field.rs` | `TableField`（derive） | |
| `TableLogic` | `table_logic.rs` | `TableLogic`（derive） | |
| `Version` | `version.rs` | `Version`（derive） | |
| `EnumValue` | `enum_value.rs` | `EnumValue`（derive） | |
| `FieldFill` | `field_fill.rs` | `FieldFill`（derive/enum） | |
| `FieldStrategy` | `field_strategy.rs` | `FieldStrategy`（derive/enum） | |
| `InterceptorIgnore` | `interceptor_ignore.rs` | `InterceptorIgnore`（derive） | |
| `KeySequence` | `key_sequence.rs` | `KeySequence`（derive） | |
| `OrderBy` | `order_by.rs` | `OrderBy`（derive） | |
| `SqlCondition` | `sql_condition.rs` | `SqlCondition`（const） | |
| `DbType` | `db_type.rs` | `DbType`（enum） | 13+ dialect |
| `IEnum` | `i_enum.rs` | `Enum`（trait，去 I 前缀） | 文件名保留 `i_enum.rs` |
| `IdType` | `id_type.rs` | `IdType`（enum） | |
| `IgnoreEncrypted` | `ignore_encrypted.rs` | `IgnoreEncrypted`（derive/attr） | 来自 enhance |

### 3.2 conditions（28）

| Java 类 | 文件名 | Rust 类型 | 说明 |
|---|---|---|---|
| `Wrapper` | `wrapper.rs` | `Wrapper<T>` | 抽象基类 |
| `AbstractWrapper` | `abstract_wrapper.rs` | `AbstractWrapper<T,R,Children>` | 核心 |
| `QueryWrapper` | `query/query_wrapper.rs` | `QueryWrapper<T>` | |
| `UpdateWrapper` | `update/update_wrapper.rs` | `UpdateWrapper<T>` | |
| `LambdaQueryWrapper` | `query/lambda_query_wrapper.rs` | `LambdaQueryWrapper<T>` | |
| `LambdaUpdateWrapper` | `update/lambda_update_wrapper.rs` | `LambdaUpdateWrapper<T>` | |
| `Compare` | `compare.rs` | `Compare<R>` | trait |
| `Func` | `func.rs` | `Func<R>` | trait |
| `Nested` | `nested.rs` | `Nested<R>` | trait |
| `Join` | `join.rs` | `Join<R>` | trait |
| `MergeSegments` | `merge_segments.rs` | `MergeSegments` | |
| `SharedString` | `shared_string.rs` | `SharedString` | |
| `ISqlSegment` | `i_sql_segment.rs` | `ISqlSegment` | |
| `NormalSegmentList` | `normal_segment_list.rs` | `NormalSegmentList` | |
| `GroupBySegmentList` | `group_by_segment_list.rs` | `GroupBySegmentList` | |
| `OrderBySegmentList` | `order_by_segment_list.rs` | `OrderBySegmentList` | |
| `HavingSegmentList` | `having_segment_list.rs` | `HavingSegmentList` | |
| `ColumnSegment` | `column_segment.rs` | `ColumnSegment` | |
| `Wrappers` | `wrappers.rs` | `Wrappers` | 工厂方法 |
| `SqlKeyword` | `sql_keyword.rs` | `SqlKeyword` | enum |
| `SqlLike` | `sql_like.rs` | `SqlLike` | enum |
| `WrapperKeyword` | `wrapper_keyword.rs` | `WrapperKeyword` | enum |
| `Constants` | `constants.rs` | `Constants` | |
| `Column<F>` | `query/column.rs` | `Column<F>` | Rust 新增 |

### 3.3 mapper（3）

| Java 类 | 文件名 | Rust 类型 | 说明 |
|---|---|---|---|
| `BaseMapper` | `base_mapper.rs` | `BaseMapper<T>` | trait |
| `Mapper` | `mapper.rs` | `Mapper<T>` | 标记 trait |
| `MapperProxyMetadata` | `mapper_proxy_metadata.rs` | `MapperProxyMetadata` | |

### 3.4 metadata（8）

| Java 类 | 文件名 | Rust 类型 | 说明 |
|---|---|---|---|
| `TableInfo` | `table_info.rs` | `TableInfo` | |
| `TableFieldInfo` | `table_field_info.rs` | `TableFieldInfo` | |
| `TableIdInfo` | `table_id_info.rs` | `TableIdInfo` | |
| `OrderFieldInfo` | `order_field_info.rs` | `OrderFieldInfo` | |
| `MetaObject` | `meta_object.rs` | `MetaObject` | |
| `ColumnCache` | `column_cache.rs` | `ColumnCache` | |
| `TableInfoHelper` | `table_info_helper.rs` | `TableInfoHelper` | |
| `TableInfoHelperFactory` | `table_info_helper_factory.rs` | `TableInfoHelperFactory` | |

### 3.5 method（21）

| Java 类 | 文件名 | Rust 类型 | 说明 |
|---|---|---|---|
| `AbstractMethod` | `abstract_method.rs` | `AbstractMethod` | |
| `Insert` | `insert.rs` | `Insert` | |
| `Delete` | `delete.rs` | `Delete` | |
| `DeleteById` | `delete_by_id.rs` | `DeleteById` | |
| `DeleteByMap` | `delete_by_map.rs` | `DeleteByMap` | |
| `DeleteByIds` | `delete_by_ids.rs` | `DeleteByIds` | |
| `DeleteBatchByIds` | `delete_batch_by_ids.rs` | `DeleteBatchByIds` | |
| `Update` | `update.rs` | `Update` | |
| `UpdateById` | `update_by_id.rs` | `UpdateById` | |
| `SelectById` | `select_by_id.rs` | `SelectById` | |
| `SelectByIds` | `select_by_ids.rs` | `SelectByIds` | |
| `SelectBatchByIds` | `select_batch_by_ids.rs` | `SelectBatchByIds` | |
| `SelectByMap` | `select_by_map.rs` | `SelectByMap` | |
| `SelectCount` | `select_count.rs` | `SelectCount` | |
| `SelectList` | `select_list.rs` | `SelectList` | |
| `SelectMaps` | `select_maps.rs` | `SelectMaps` | |
| `SelectMapsPage` | `select_maps_page.rs` | `SelectMapsPage` | |
| `SelectObjs` | `select_objs.rs` | `SelectObjs` | |
| `SelectOne` | `select_one.rs` | `SelectOne` | |
| `SelectPage` | `select_page.rs` | `SelectPage` | |
| `SelectWithCursor` | `select_with_cursor.rs` | `SelectWithCursor` | |

---

## 4. 一文件一对象规则

每个 `.rs` 文件必须只对应一个 Java 对象。`mod.rs` 只做模块声明与 re-export，**禁止定义任何类型/逻辑**。`lib.rs` 只做 crate 门面，**禁止堆放对象**。不允许出现 `compat.rs`。

---

## 5. 注释规范

每个文件头部必须有中文 doc 注释：
- 说明对应 Java 类的全限定名
- 核心职责
- 与 Java 实现的差异
- 方法级中文注释从 Java 源码同步翻译
