/// 表字段属性（由 `#[derive(TableField)]` 宏生成）。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.TableField`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/TableField.java`
///
/// 此结构体同时承载 `@TableField` 注解属性，以及 `@Version` 和 `@TableLogic` 注解的运行时状态。
/// 在 Java 中，这些分属不同注解；Rust 端合并到一个结构体，由 derive 宏在编译期统一填充。
///
/// # 与 Java 字段的对应关系
///
/// | Java `@TableField` 属性 | Rust 字段 | 默认值 | 说明 |
/// |---|---|---|---|
/// | `value()` | `column` | `""` | 数据库列名 |
/// | `exist()` | `exist` | `true` | 是否为数据库表字段 |
/// | `condition()` | `condition` | `""` | WHERE 条件（对应 `SqlCondition.EQUAL`） |
/// | `update()` | `update` | `""` | UPDATE SET 注入表达式（如 `"%s+1"`、`"now()"`) |
/// | `insertStrategy()` | `insert_strategy` | `DEFAULT` | INSERT 字段策略 |
/// | `updateStrategy()` | `update_strategy` | `DEFAULT` | UPDATE 字段策略 |
/// | `whereStrategy()` | `where_strategy` | `DEFAULT` | WHERE 条件策略 |
/// | `fill()` | `fill` | `DEFAULT` | 自动填充策略 |
/// | `select()` | `select` | `true` | 是否参与 SELECT |
/// | `keepGlobalFormat()` | `keep_global_format` | `false` | 是否保持全局 columnFormat |
/// | `property()` | `property` | `""` | ResultMapping/ParameterMapping 属性名 |
/// | `jdbcType()` | `jdbc_type` | `"UNDEFINED"` | JDBC 类型（字符串形式） |
/// | `typeHandler()` | `type_handler` | `""` | 类型处理器名称（字符串形式） |
/// | `javaType()` | `java_type` | `false` | 是否辅助追加 javaType |
/// | `numericScale()` | `numeric_scale` | `""` | 小数点后保留位数 |
///
/// 另外携带 `@Version` 和 `@TableLogic` 的运行时信息：
/// | 注解 | Rust 字段 | 说明 |
/// |---|---|---|
/// | `@Version` | `version` | 是否为乐观锁字段 |
/// | `@TableLogic` | `logic_delete` | 是否为逻辑删除字段 |
/// | `@TableLogic.value()` | `logic_not_delete_value` | 逻辑未删除值 |
/// | `@TableLogic.delval()` | `logic_delete_value` | 逻辑删除值 |
use super::field_fill::FieldFill;
use super::field_strategy::FieldStrategy;

/// 表字段属性（由 `#[derive(TableField)]` 宏生成）。
///
/// 完整对应 Java `@TableField` 16 个注解属性 + `@Version` / `@TableLogic` 运行时状态。
#[derive(Debug, Clone)]
pub struct TableFieldAttr {
    /// Rust 属性名（如 `my_name`），由 derive 宏从 struct 字段名自动填充。
    pub property: &'static str,
    /// 数据库列名（对应 `@TableField.value()`；默认空串表示由全局配置自动映射）。
    pub column: &'static str,
    /// 是否为数据库表字段（对应 `@TableField.exist()`；默认 true）。
    pub exist: bool,
    /// WHERE 条件表达式（对应 `@TableField.condition()`；默认空串表示使用 `SqlCondition::EQUAL`）。
    pub condition: &'static str,
    /// INSERT 字段策略（对应 `@TableField.insertStrategy()`；默认 `DEFAULT`）。
    pub insert_strategy: FieldStrategy,
    /// UPDATE 字段策略（对应 `@TableField.updateStrategy()`；默认 `DEFAULT`）。
    pub update_strategy: FieldStrategy,
    /// WHERE 字段策略（对应 `@TableField.whereStrategy()`；默认 `DEFAULT`）。
    pub where_strategy: FieldStrategy,
    /// 自动填充策略（对应 `@TableField.fill()`；默认 `DEFAULT`）。
    pub fill: FieldFill,
    /// 是否参与 SELECT 查询（对应 `@TableField.select()`；默认 true）。
    pub select: bool,
    /// 是否保持全局 columnFormat（对应 `@TableField.keepGlobalFormat()`；默认 false）。
    pub keep_global_format: bool,
    /// UPDATE SET 注入表达式（对应 `@TableField.update()`；默认空串）。
    pub update: &'static str,
    /// JDBC 类型名称（对应 `@TableField.jdbcType()`；Rust 用字符串代替 Java `JdbcType` 枚举）。
    pub jdbc_type: &'static str,
    /// 类型处理器类名（对应 `@TableField.typeHandler()`；Rust 用字符串代替 Java `Class<?>` 泛型）。
    pub type_handler: &'static str,
    /// 是否辅助追加 javaType（对应 `@TableField.javaType()`；默认 false）。
    pub java_type: bool,
    /// 小数点后保留位数（对应 `@TableField.numericScale()`；默认空串表示不限制）。
    pub numeric_scale: &'static str,
    /// ResultMapping/ParameterMapping 属性名（对应 `@TableField.property()`）。
    /// 当 `property` 非空时，用于 XML 映射的 property 绑定。
    pub result_property: &'static str,
    /// 是否为乐观锁字段（由 `#[derive(Version)]` 注解填充；与 `@TableField` 无直接关系，但运行时需要）。
    pub version: bool,
    /// 是否为逻辑删除字段（由 `#[derive(TableLogic)]` 注解填充）。
    pub logic_delete: bool,
    /// 逻辑未删除值（由 `#[derive(TableLogic)]` 注解填充；默认空串表示用全局配置）。
    pub logic_not_delete_value: &'static str,
    /// 逻辑删除值（由 `#[derive(TableLogic)]` 注解填充）。
    pub logic_delete_value: &'static str,
}

impl Default for TableFieldAttr {
    fn default() -> Self {
        Self {
            property: "",
            column: "",
            exist: true,
            condition: "",
            insert_strategy: FieldStrategy::default(),
            update_strategy: FieldStrategy::default(),
            where_strategy: FieldStrategy::default(),
            fill: FieldFill::default(),
            select: true,
            keep_global_format: false,
            update: "",
            jdbc_type: "UNDEFINED",
            type_handler: "",
            java_type: false,
            numeric_scale: "",
            result_property: "",
            version: false,
            logic_delete: false,
            logic_not_delete_value: "",
            logic_delete_value: "",
        }
    }
}

impl TableFieldAttr {
    /// 从 `@TableField` 注解属性构建（Rust proc-macro 展开时调用）。
    ///
    /// 对应 Java：`AnnotationUtils.findAnnotation(field, TableField.class)` → 提取注解属性值。
    pub fn builder() -> TableFieldAttrBuilder {
        TableFieldAttrBuilder::default()
    }
}

/// `TableFieldAttr` 构建器，用于 proc-macro 展开时构造属性。
///
/// **默认值与 Java `@TableField` 一致**：`exist` = true、`select` = true，与 `TableFieldAttr::default()` 对齐。
pub struct TableFieldAttrBuilder {
    property: &'static str,
    column: &'static str,
    exist: bool,
    condition: &'static str,
    insert_strategy: FieldStrategy,
    update_strategy: FieldStrategy,
    where_strategy: FieldStrategy,
    fill: FieldFill,
    select: bool,
    keep_global_format: bool,
    update: &'static str,
    jdbc_type: &'static str,
    type_handler: &'static str,
    java_type: bool,
    numeric_scale: &'static str,
    result_property: &'static str,
    version: bool,
    logic_delete: bool,
    logic_not_delete_value: &'static str,
    logic_delete_value: &'static str,
}

impl Default for TableFieldAttrBuilder {
    fn default() -> Self {
        Self {
            property: "",
            column: "",
            exist: true,         // Java @TableField.exist() 默认 true
            condition: "",
            insert_strategy: FieldStrategy::default(),
            update_strategy: FieldStrategy::default(),
            where_strategy: FieldStrategy::default(),
            fill: FieldFill::default(),
            select: true,        // Java @TableField.select() 默认 true
            keep_global_format: false,
            update: "",
            jdbc_type: "UNDEFINED",
            type_handler: "",
            java_type: false,
            numeric_scale: "",
            result_property: "",
            version: false,
            logic_delete: false,
            logic_not_delete_value: "",
            logic_delete_value: "",
        }
    }
}

impl TableFieldAttrBuilder {
    pub fn property(mut self, v: &'static str) -> Self { self.property = v; self }
    pub fn column(mut self, v: &'static str) -> Self { self.column = v; self }
    pub fn exist(mut self, v: bool) -> Self { self.exist = v; self }
    pub fn condition(mut self, v: &'static str) -> Self { self.condition = v; self }
    pub fn insert_strategy(mut self, v: FieldStrategy) -> Self { self.insert_strategy = v; self }
    pub fn update_strategy(mut self, v: FieldStrategy) -> Self { self.update_strategy = v; self }
    pub fn where_strategy(mut self, v: FieldStrategy) -> Self { self.where_strategy = v; self }
    pub fn fill(mut self, v: FieldFill) -> Self { self.fill = v; self }
    pub fn select(mut self, v: bool) -> Self { self.select = v; self }
    pub fn keep_global_format(mut self, v: bool) -> Self { self.keep_global_format = v; self }
    pub fn update(mut self, v: &'static str) -> Self { self.update = v; self }
    pub fn jdbc_type(mut self, v: &'static str) -> Self { self.jdbc_type = v; self }
    pub fn type_handler(mut self, v: &'static str) -> Self { self.type_handler = v; self }
    pub fn java_type(mut self, v: bool) -> Self { self.java_type = v; self }
    pub fn numeric_scale(mut self, v: &'static str) -> Self { self.numeric_scale = v; self }
    pub fn result_property(mut self, v: &'static str) -> Self { self.result_property = v; self }
    pub fn version(mut self, v: bool) -> Self { self.version = v; self }
    pub fn logic_delete(mut self, v: bool) -> Self { self.logic_delete = v; self }
    pub fn logic_not_delete_value(mut self, v: &'static str) -> Self { self.logic_not_delete_value = v; self }
    pub fn logic_delete_value(mut self, v: &'static str) -> Self { self.logic_delete_value = v; self }

    pub fn build(self) -> TableFieldAttr {
        TableFieldAttr {
            property: self.property,
            column: self.column,
            exist: self.exist,
            condition: self.condition,
            insert_strategy: self.insert_strategy,
            update_strategy: self.update_strategy,
            where_strategy: self.where_strategy,
            fill: self.fill,
            select: self.select,
            keep_global_format: self.keep_global_format,
            update: self.update,
            jdbc_type: self.jdbc_type,
            type_handler: self.type_handler,
            java_type: self.java_type,
            numeric_scale: self.numeric_scale,
            result_property: self.result_property,
            version: self.version,
            logic_delete: self.logic_delete,
            logic_not_delete_value: self.logic_not_delete_value,
            logic_delete_value: self.logic_delete_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_java() {
        let attr = TableFieldAttr::default();
        assert_eq!(attr.column, "");
        assert!(attr.exist);
        assert_eq!(attr.insert_strategy, FieldStrategy::default());
        assert_eq!(attr.update_strategy, FieldStrategy::default());
        assert_eq!(attr.where_strategy, FieldStrategy::default());
        assert_eq!(attr.fill, FieldFill::default());
        assert!(attr.select);
        assert!(!attr.keep_global_format);
        assert!(!attr.java_type);
        assert!(!attr.version);
        assert!(!attr.logic_delete);
        assert_eq!(attr.jdbc_type, "UNDEFINED");
        assert_eq!(attr.numeric_scale, "");
    }

    #[test]
    fn builder_sets_all_fields() {
        let attr = TableFieldAttr::builder()
            .column("user_name")
            .property("name")
            .exist(true)
            .condition("=` ?")
            .insert_strategy(FieldStrategy::NotNull)
            .update_strategy(FieldStrategy::NotEmpty)
            .where_strategy(FieldStrategy::Always)
            .fill(FieldFill::InsertUpdate)
            .select(true)
            .keep_global_format(true)
            .update("%s+1")
            .jdbc_type("VARCHAR")
            .type_handler("com.example.MyTypeHandler")
            .java_type(true)
            .numeric_scale("2")
            .result_property("userName")
            .version(true)
            .logic_delete(true)
            .logic_not_delete_value("0")
            .logic_delete_value("1")
            .build();

        assert_eq!(attr.column, "user_name");
        assert_eq!(attr.property, "name");
        assert!(attr.exist);
        assert_eq!(attr.condition, "=` ?");
        assert_eq!(attr.insert_strategy, FieldStrategy::NotNull);
        assert_eq!(attr.update_strategy, FieldStrategy::NotEmpty);
        assert_eq!(attr.where_strategy, FieldStrategy::Always);
        assert_eq!(attr.fill, FieldFill::InsertUpdate);
        assert!(attr.select);
        assert!(attr.keep_global_format);
        assert_eq!(attr.update, "%s+1");
        assert_eq!(attr.jdbc_type, "VARCHAR");
        assert_eq!(attr.type_handler, "com.example.MyTypeHandler");
        assert!(attr.java_type);
        assert_eq!(attr.numeric_scale, "2");
        assert_eq!(attr.result_property, "userName");
        assert!(attr.version);
        assert!(attr.logic_delete);
        assert_eq!(attr.logic_not_delete_value, "0");
        assert_eq!(attr.logic_delete_value, "1");
    }

    #[test]
    fn builder_produces_default_when_no_override() {
        let attr = TableFieldAttr::builder().column("id").build();
        // builder 默认值与 Java @TableField 注解默认值一致
        assert!(attr.exist);      // Java @TableField.exist() 默认 true
        assert!(attr.select);     // Java @TableField.select() 默认 true
        assert!(!attr.keep_global_format);
        assert!(!attr.java_type);
        assert!(!attr.version);
        assert!(!attr.logic_delete);
    }
}
