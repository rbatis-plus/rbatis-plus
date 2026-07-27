//! SQL 注入方法基类（对标 Java `AbstractMethod`）。
//!
//! 对应 Java：`com.baomidou.mybatisplus.core.injector.AbstractMethod`
//! 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/injector/AbstractMethod.java`

use crate::metadata::TableInfo;

/// SQL 方法生成结果：包含 SQL 模板字符串和相关的元信息。
///
/// 对应 Java：`MappedStatement` 的简化版本（Rust 端不依赖 MyBatis 容器）。
#[derive(Debug, Clone)]
pub struct MethodResult {
    /// SQL 语句模板（包含 `?` 占位符，由 rbatis 的 Prepared Statement 绑定）。
    pub sql: String,
    /// SQL 方法名（用于 Mapper 方法映射，如 `"insert"`、`"selectList"`）。
    pub method_name: String,
    /// 主键列名（用于 Jdbc3KeyGenerator 自增主键回填，仅 INSERT 时有值）。
    pub key_column: Option<String>,
    /// 主键属性名（用于回填到实体对象）。
    pub key_property: Option<String>,
}

/// SQL 注入方法 trait（对标 Java `AbstractMethod`）。
///
/// 每个方法类实现此 trait，提供从 `TableInfo` 生成 SQL 模板的能力。
/// SQL 模板中的 `?` 占位符由 rbatis 的 Prepared Statement 在运行时绑定。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.AbstractMethod`
pub trait AbstractMethod: Send + Sync + std::fmt::Debug {
    /// 生成 SQL 模板。
    ///
    /// 对应 Java：`AbstractMethod.injectMappedStatement(...)` 中的 SQL 拼装部分。
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::TableFieldInfo;
    use crate::derive::FieldStrategy;

    fn test_table_info() -> TableInfo {
        TableInfo {
            entity_type: "User",
            table_name: "users".into(),
            key_column: "id".into(),
            key_property: "id".into(),
            id_type: crate::derive::IdType::Auto,
            field_list: vec![
                TableFieldInfo { column: "name".into(), property: "name".into(), insert_strategy: FieldStrategy::NotNull, ..Default::default() },
                TableFieldInfo { column: "email".into(), property: "email".into(), insert_strategy: FieldStrategy::NotEmpty, ..Default::default() },
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
        }
    }

    #[test]
    fn method_result_creation() {
        let result = MethodResult {
            sql: "INSERT INTO users (id, name) VALUES (#{id}, #{name})".into(),
            method_name: "insert".into(),
            key_column: Some("id".into()),
            key_property: Some("id".into()),
        };
        assert_eq!(result.sql, "INSERT INTO users (id, name) VALUES (#{id}, #{name})");
        assert_eq!(result.key_column.as_deref(), Some("id"));
    }

    #[test]
    fn method_result_no_pk() {
        let result = MethodResult {
            sql: "SELECT COUNT(*) AS total FROM users".into(),
            method_name: "selectCount".into(),
            key_column: None,
            key_property: None,
        };
        assert!(result.key_column.is_none());
        assert!(result.key_property.is_none());
    }
}
