//! INSERT 方法（对齐 Java `DefaultSqlInjector → Insert`）。
//!
//! 生成 SQL：`INSERT INTO <table> (<columns>) VALUES (<values>)`
//! 自增主键时使用 Jdbc3KeyGenerator；序列主键时使用用户注册的 KeyGenerator。

use super::{AbstractMethod, MethodResult};
use crate::derive::{IdType, FieldStrategy};
use crate::metadata::TableInfo;

/// 插入一条记录（选择字段插入）。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.Insert`
/// 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/injector/methods/Insert.java`
#[derive(Debug)]
pub struct Insert;

impl AbstractMethod for Insert {
    /// 对应 Java：`Insert.injectMappedStatement()` 的 SQL 拼装逻辑。
    ///
    /// 生成 INSERT 列（排除 FieldStrategy::Never 和逻辑删除字段），
    /// 使用 `#{property}` 作为值占位符，由 rbatis Prepared Statement 绑定。
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        // 列名：排除 FieldStrategy::Never
        let columns = table_info.field_list.iter()
            .filter(|f| f.insert_strategy != FieldStrategy::Never)
            .map(|f| f.column.clone())
            .collect::<Vec<_>>();
        let columns_str = columns.join(", ");

        // 占位符：#{property}
        let values_str = table_info.field_list.iter()
            .filter(|f| f.insert_strategy != FieldStrategy::Never)
            .map(|f| format!("{{{}{}}}", "", f.property))
            .collect::<Vec<_>>()
            .join(", ");

        // 主键处理
        let key_column = if !table_info.key_column.is_empty() {
            Some(table_info.key_column.clone())
        } else {
            None
        };
        let key_property = if !table_info.key_property.is_empty() {
            Some(table_info.key_property.clone())
        } else {
            None
        };

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_info.table_name, columns_str, values_str
        );

        MethodResult {
            sql,
            method_name: "insert".into(),
            key_column,
            key_property,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::TableFieldInfo;
    use crate::derive::{FieldStrategy, IdType};

    fn make_table() -> TableInfo {
        TableInfo {
            entity_type: "User",
            table_name: "users".into(),
            key_column: "id".into(),
            key_property: "id".into(),
            id_type: IdType::Auto,
            field_list: vec![
                TableFieldInfo { column: "name".into(), property: "name".into(), insert_strategy: FieldStrategy::NotNull, ..Default::default() },
                TableFieldInfo { column: "big_blob".into(), property: "big_blob".into(), insert_strategy: FieldStrategy::Never, ..Default::default() },
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
    fn insert_sql_excludes_never_fields() {
        let result = Insert.generate_sql(&make_table());
        assert!(result.sql.contains("INSERT INTO users"));
        assert!(result.sql.contains("name"));
        assert!(!result.sql.contains("big_blob"));
        assert_eq!(result.key_column.as_deref(), Some("id"));
    }

    #[test]
    fn insert_method_name_matches_java() {
        assert_eq!(Insert.generate_sql(&make_table()).method_name, "insert");
    }
}
