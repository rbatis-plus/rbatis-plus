//! SELECT 列表（`SELECT <columns> FROM <table> <where>`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按 Wrapper 条件查询列表。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.SelectList`
#[derive(Debug)]
pub struct SelectList;

impl AbstractMethod for SelectList {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let columns = table_info.all_sql_select();
        MethodResult {
            sql: format!("SELECT {} FROM {}", columns, table_info.table_name),
            method_name: "selectList".into(),
            key_column: None,
            key_property: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::test_utils::user_table_info;

    #[test]
    fn select_list_sql() {
        let result = SelectList.generate_sql(&user_table_info());
        assert!(result.sql.contains("SELECT"));
        assert!(result.sql.contains("FROM users"));
        assert_eq!(result.key_column, None);
    }
}
