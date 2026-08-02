//! 按主键删除（`DELETE FROM <table> WHERE <pk> = ?`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按主键删除一条记录。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.DeleteById`
#[derive(Debug)]
pub struct DeleteById;

impl AbstractMethod for DeleteById {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        MethodResult {
            sql: format!("DELETE FROM {} WHERE {} = ?", table_info.table_name, table_info.key_column),
            method_name: "deleteById".into(),
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
    fn delete_by_id_sql() {
        let result = DeleteById.generate_sql(&user_table_info());
        assert!(result.sql.contains("DELETE FROM users"));
        assert!(result.sql.contains("WHERE id ="));
        assert_eq!(result.key_column.as_deref(), Some("id"));
    }
}
