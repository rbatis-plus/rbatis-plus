//! SELECT 单列值（`SELECT <first_column> FROM <table> <where>`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按 Wrapper 条件查询，只返回第一列值（`List<E>`，E 为第一列类型）。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.SelectObjs`
#[derive(Debug)]
pub struct SelectObjs;

impl AbstractMethod for SelectObjs {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        // 只选第一列：通常为主键或 id
        let first_col = table_info.key_column.clone();
        MethodResult {
            sql: format!("SELECT {} FROM {}", first_col, table_info.table_name),
            method_name: "selectObjs".into(),
            key_column: None,
            key_property: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::test_utils::test_utils::user_table_info;

    #[test]
    fn select_objs_first_column() {
        let result = SelectObjs.generate_sql(&user_table_info());
        // 应该只选第一列（pk）
        assert!(result.sql.contains("SELECT id"));
        assert!(!result.sql.contains("name"));
        assert!(!result.sql.contains("email"));
    }
}
