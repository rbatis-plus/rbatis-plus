//! INSERT 方法（`INSERT INTO <table> (<columns>) VALUES (<values>)`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 插入一条记录（选择字段插入）。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.Insert`
#[derive(Debug)]
pub struct Insert;

impl AbstractMethod for Insert {
    /// 生成 INSERT SQL 模板。
    ///
    /// 占位符格式为 `?`（SQL 参数绑定），由 rbatis 在运行时绑定。
    ///
    /// 对应 Java：`Insert.injectMappedStatement()` 的 SQL 拼装逻辑。
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let columns = table_info.field_list.iter()
            .filter(|f| f.insert_strategy != crate::derive::FieldStrategy::Never)
            .map(|f| f.column.clone())
            .collect::<Vec<_>>();
        let columns_str = columns.join(", ");

        let placeholders: Vec<String> = columns.iter().map(|_| "?".to_string()).collect();
        let values_str = placeholders.join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table_info.table_name, columns_str, values_str
        );

        MethodResult {
            sql,
            method_name: "insert".into(),
            key_column: if !table_info.key_column.is_empty() {
                Some(table_info.key_column.clone())
            } else {
                None
            },
            key_property: if !table_info.key_property.is_empty() {
                Some(table_info.key_property.clone())
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::test_utils::user_table_info;

    #[test]
    fn insert_sql_excludes_never_fields() {
        let result = Insert.generate_sql(&user_table_info());
        assert!(result.sql.contains("INSERT INTO users"));
        assert!(result.sql.contains("name"));
        assert!(!result.sql.contains("big_blob"));
        assert_eq!(result.key_column.as_deref(), Some("id"));
        assert!(result.sql.contains("?"));
    }
}
