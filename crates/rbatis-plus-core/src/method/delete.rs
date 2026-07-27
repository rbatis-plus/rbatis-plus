//! DELETE 方法（`DELETE FROM <table>`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按 Wrapper 条件删除（`DELETE FROM <table> <where>`）。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.Delete`
#[derive(Debug)]
pub struct Delete;

impl AbstractMethod for Delete {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        MethodResult {
            sql: format!("DELETE FROM {}", table_info.table_name),
            method_name: "delete".into(),
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
    fn delete_sql() {
        let result = Delete.generate_sql(&user_table_info());
        assert!(result.sql.contains("DELETE FROM users"));
        assert_eq!(result.method_name, "delete");
        assert!(result.key_column.is_none());
    }
}
