// Source: mybatis-plus-core/.../metadata/TableInfo.java
// Source: mybatis-plus-core/.../metadata/TableFieldInfo.java

use crate::derive::{
    FieldFill, FieldStrategy, IdType, TableFieldAttr,
};

/// Per-entity table metadata.
///
/// 对应 Java：`com.baomidou.mybatisplus.core.metadata.TableInfo`
/// 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/metadata/TableInfo.java`
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// The entity type name (Rust struct name).
    pub entity_type: &'static str,
    /// The database table name.
    pub table_name: String,
    /// Primary key column name.
    pub key_column: String,
    /// Primary key property (Rust field) name.
    pub key_property: String,
    /// Primary key generation strategy.
    pub id_type: IdType,
    /// All field infos (excluding the PK).
    pub field_list: Vec<TableFieldInfo>,
    /// Whether this entity has a logic-delete field.
    pub with_logic_delete: bool,
    /// The logic-delete field info (if any).
    pub logic_delete_field: Option<TableFieldInfo>,
    /// Whether this entity has a version field.
    pub with_version: bool,
    /// The version field info (if any).
    pub version_field: Option<TableFieldInfo>,
    /// Whether to auto-initialize ResultMap (对应 Java `autoInitResultMap`）。
    pub auto_init_result_map: bool,
    /// Whether the key is related to another column (对应 Java `keyRelated`）。
    pub key_related: bool,
    /// Global column format（对应 Java `columnFormat`，如反引号包围列名）。
    pub column_format: String,
    /// Whether this table uses underCamel mapping（对应 Java `underCamel`）。
    pub under_camel: bool,
    /// Result set ordered flag（对应 Java `resultOrdered`）。
    pub result_ordered: bool,
    /// Ordered fields list（对应 Java `orderByFields`）。
    pub order_by_fields: Vec<String>,
}

impl TableInfo {
    /// Build the SELECT column list (all fields, comma-separated).
    pub fn all_sql_select(&self) -> String {
        let mut cols = vec![self.key_column.clone()];
        for f in &self.field_list {
            if f.select {
                cols.push(f.column.clone());
            }
        }
        cols.join(", ")
    }

    /// Whether the entity has a primary key.
    pub fn have_pk(&self) -> bool {
        !self.key_column.is_empty()
    }

    /// Build the INSERT column list (all non-null fields).
    pub fn all_insert_sql_column(&self, prefix: &str) -> String {
        let _ = prefix;
        let mut cols = vec![self.key_column.clone()];
        for f in &self.field_list {
            if f.insert_strategy != FieldStrategy::Never {
                cols.push(f.column.clone());
            }
        }
        cols.join(", ")
    }

    /// Build the WHERE PK fragment: `key_column = #{prefix.key_property}`.
    pub fn get_sql_where(&self, prefix: &str) -> String {
        format!("{} = {{{}{}}}", self.key_column, prefix, self.key_property)
    }
}

/// Per-field metadata.
///
/// 对应 Java：`com.baomidou.mybatisplus.core.metadata.TableFieldInfo`
/// 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/metadata/TableFieldInfo.java`
///
/// Java 3.5.17 TableFieldInfo 包含 18+ 个字段；Rust 端逐一映射，
/// 其中 `JdbcType` 用 `String` 替代（Rust 无对应 JDBC 枚举），
/// `TypeHandler` 用 `String` 替代（Rust 无 `Class<?>` 泛型）。
#[derive(Debug, Clone)]
pub struct TableFieldInfo {
    /// The database column name.
    pub column: String,
    /// The Rust property (field) name.
    pub property: String,
    /// The el expression (for XML-style mapping, usually empty in Rust).
    pub el: String,
    /// Insert strategy.
    pub insert_strategy: FieldStrategy,
    /// Update strategy.
    pub update_strategy: FieldStrategy,
    /// Where strategy.
    pub where_strategy: FieldStrategy,
    /// Auto-fill behaviour.
    pub fill: FieldFill,
    /// Whether to include this field in SELECT.
    pub select: bool,
    /// Whether this is the version field (optimistic lock).
    pub version: bool,
    /// Whether this is the logic-delete field.
    pub logic_delete: bool,
    /// The value representing "not deleted".
    pub logic_not_delete_value: String,
    /// The value representing "deleted".
    pub logic_delete_value: String,
    /// Raw SET expression for update (e.g. `"now()"` or `"%s+1"`).
    pub update: String,
    /// WHERE condition expression（对应 Java `TableFieldInfo.condition`）。
    pub condition: String,
    /// Whether to keep global columnFormat（对应 Java `TableFieldInfo.keepGlobalFormat`）。
    pub keep_global_format: bool,
    /// JDBC type name（对应 Java `JdbcType` 枚举；Rust 用字符串 "VARCHAR"/"UNDEFINED" 等）。
    pub jdbc_type: String,
    /// Type handler class name（对应 Java `Class<? extends TypeHandler>`）。
    pub type_handler: String,
    /// Whether to include javaType（对应 Java `TableFieldInfo.javaType`）。
    pub java_type: bool,
    /// Numeric scale for decimal fields（对应 Java `TableFieldInfo.numericScale`）。
    pub numeric_scale: String,
    /// ResultMapping/ParameterMapping property name（对应 Java `TableFieldInfo.property`，与 `property` 字段相同但用于映射）。
    pub result_property: String,
    /// Is this field primitive type（对应 Java `TableFieldInfo.isPrimitive`）。
    pub is_primitive: bool,
    /// Is this field a char sequence type（对应 Java `TableFieldInfo.isCharSequence`）。
    pub is_char_sequence: bool,
    /// SQL select fragment for this field（对应 Java `TableFieldInfo.sqlSelect`）。
    pub sql_select: String,
    /// Is this field ordered（对应 Java `TableFieldInfo.isOrderBy`）。
    pub is_order_by: bool,
    /// Order by type（对应 Java `TableFieldInfo.orderByType`）。
    pub order_by_type: String,
    /// Order by sort priority（对应 Java `TableFieldInfo.orderBySort`）。
    pub order_by_sort: i16,
}

impl TableFieldInfo {
    /// Create from a `TableFieldAttr` (annotation descriptor).
    ///
    /// 对应 Java：`TableFieldInfo` 构造器 + `AnnotationUtils.findAnnotation` 调用。
    pub fn from_attr(attr: &TableFieldAttr) -> Self {
        let is_char_sequence = matches!(
            attr.property,
            n if n.contains("String") || n.contains("str") || n.contains("text")
        );
        Self {
            column: attr.column.to_string(),
            property: attr.property.to_string(),
            el: String::new(),
            insert_strategy: attr.insert_strategy,
            update_strategy: attr.update_strategy,
            where_strategy: attr.where_strategy,
            fill: attr.fill,
            select: attr.select,
            version: attr.version,
            logic_delete: attr.logic_delete,
            logic_not_delete_value: attr.logic_not_delete_value.to_string(),
            logic_delete_value: attr.logic_delete_value.to_string(),
            update: attr.update.to_string(),
            condition: attr.condition.to_string(),
            keep_global_format: attr.keep_global_format,
            jdbc_type: attr.jdbc_type.to_string(),
            type_handler: attr.type_handler.to_string(),
            java_type: attr.java_type,
            numeric_scale: attr.numeric_scale.to_string(),
            result_property: attr.result_property.to_string(),
            is_primitive: false,
            is_char_sequence,
            sql_select: String::new(),
            is_order_by: false,
            order_by_type: String::new(),
            order_by_sort: 0,
        }
    }
}

impl Default for TableFieldInfo {
    fn default() -> Self {
        Self {
            column: String::new(),
            property: String::new(),
            el: String::new(),
            insert_strategy: FieldStrategy::default(),
            update_strategy: FieldStrategy::default(),
            where_strategy: FieldStrategy::default(),
            fill: FieldFill::default(),
            select: true,
            version: false,
            logic_delete: false,
            logic_not_delete_value: String::new(),
            logic_delete_value: String::new(),
            update: String::new(),
            condition: String::new(),
            keep_global_format: false,
            jdbc_type: "UNDEFINED".to_string(),
            type_handler: String::new(),
            java_type: false,
            numeric_scale: String::new(),
            result_property: String::new(),
            is_primitive: false,
            is_char_sequence: false,
            sql_select: String::new(),
            is_order_by: false,
            order_by_type: String::new(),
            order_by_sort: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_field_info_from_attr() {
        let attr = TableFieldAttr::builder()
            .column("user_name")
            .property("name")
            .condition("= ?")
            .jdbc_type("VARCHAR")
            .type_handler("MyHandler")
            .java_type(true)
            .numeric_scale("2")
            .result_property("userName")
            .build();

        let info = TableFieldInfo::from_attr(&attr);
        assert_eq!(info.column, "user_name");
        assert_eq!(info.property, "name");
        assert_eq!(info.condition, "= ?");
        assert_eq!(info.jdbc_type, "VARCHAR");
        assert_eq!(info.type_handler, "MyHandler");
        assert!(info.java_type);
        assert_eq!(info.numeric_scale, "2");
        assert_eq!(info.result_property, "userName");
        assert!(info.select);
        assert!(!info.version);
        assert!(!info.logic_delete);
    }

    #[test]
    fn table_field_info_all_sql_select() {
        let info = TableInfo {
            entity_type: "User",
            table_name: "sys_user".into(),
            key_column: "id".into(),
            key_property: "id".into(),
            id_type: IdType::None,
            field_list: vec![
                TableFieldInfo { column: "name".into(), select: true, ..Default::default() },
                TableFieldInfo { column: "big_blob".into(), select: false, ..Default::default() },
                TableFieldInfo { column: "email".into(), select: true, ..Default::default() },
            ],
            with_logic_delete: false,
            logic_delete_field: None,
            with_version: false,
            version_field: None,
            auto_init_result_map: false,
            key_related: false,
            column_format: String::new(),
            under_camel: false,
            result_ordered: false,
            order_by_fields: vec![],
        };
        assert_eq!(info.all_sql_select(), "id, name, email");
    }
}
