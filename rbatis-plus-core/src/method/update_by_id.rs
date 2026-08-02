//! 按主键更新（`UPDATE <table> SET <column> = ? WHERE <pk> = ?`）。
use super::{AbstractMethod, MethodResult};
use crate::derive::FieldStrategy;
use crate::metadata::TableInfo;

/// 按主键更新有值字段。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.UpdateById`
#[derive(Debug)]
pub struct UpdateById;

impl AbstractMethod for UpdateById {
    /// 生成 SQL 模板。
    ///
    /// 生成 `UPDATE <table> SET <column> = ?, <column2> = ? WHERE <pk> = ?`。
    /// 排除 FieldStrategy::Never 和逻辑删除字段。
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let set_clauses: Vec<String> = table_info.field_list.iter()
            .filter(|f| f.insert_strategy != FieldStrategy::Never && !f.logic_delete)
            .map(|f| format!("{} = ?", f.column))
            .collect();
        let set_sql = set_clauses.join(", ");

        MethodResult {
            sql: format!(
                "UPDATE {} SET {} WHERE {} = ?",
                table_info.table_name, set_sql, table_info.key_column
            ),
            method_name: "updateById".into(),
            key_column: Some(table_info.key_column.clone()),
            key_property: Some(table_info.key_property.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::test_utils::user_table_info;

    #[test]
    fn update_by_id_sql() {
        let result = UpdateById.generate_sql(&user_table_info());
        assert!(result.sql.contains("UPDATE users SET"));
        assert!(result.sql.contains("name = ?"));
        assert!(result.sql.contains("WHERE id = ?"));
        assert!(!result.sql.contains("big_blob"));
        assert_eq!(result.key_column.as_deref(), Some("id"));
    }
}
