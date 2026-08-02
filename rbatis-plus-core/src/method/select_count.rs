//! COUNT 查询（`SELECT COUNT(*) AS total FROM <table> <where>`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 查询总记录数。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.SelectCount`
#[derive(Debug)]
pub struct SelectCount;

impl AbstractMethod for SelectCount {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        MethodResult {
            sql: format!("SELECT COUNT(*) AS total FROM {}", table_info.table_name),
            method_name: "selectCount".into(),
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
    fn select_count_sql() {
        let result = SelectCount.generate_sql(&user_table_info());
        assert!(result.sql.contains("SELECT COUNT(*) AS total"));
        assert!(result.sql.contains("FROM users"));
    }
}
