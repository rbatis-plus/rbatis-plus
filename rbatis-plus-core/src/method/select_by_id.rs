//! 按主键查询（`SELECT <columns> FROM <table> WHERE <pk> = #{id}`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按主键查询一条记录。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.SelectById`
#[derive(Debug)]
pub struct SelectById;

impl AbstractMethod for SelectById {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let columns = table_info.all_sql_select();
        MethodResult {
            sql: format!("SELECT {} FROM {} WHERE {} = ?", columns, table_info.table_name, table_info.key_column),
            method_name: "selectById".into(),
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
    fn select_by_id_sql() {
        let result = SelectById.generate_sql(&user_table_info());
        assert!(result.sql.contains("SELECT"));
        assert!(result.sql.contains("FROM users"));
        assert!(result.sql.contains("WHERE id ="));
    }
}
