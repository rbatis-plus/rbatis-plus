//! 按主键更新（`UPDATE <table> SET <set> WHERE <pk> = #{id}`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按主键更新有值字段。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.UpdateById`
#[derive(Debug)]
pub struct UpdateById;

impl AbstractMethod for UpdateById {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let pk_col = &table_info.key_column;
        let pk_prop = &table_info.key_property;
        MethodResult {
            sql: format!("UPDATE {} SET", table_info.table_name),
            method_name: "updateById".into(),
            key_column: Some(pk_col.clone()),
            key_property: Some(pk_prop.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::test_utils::test_utils::user_table_info;

    #[test]
    fn update_by_id_sql() {
        let result = UpdateById.generate_sql(&user_table_info());
        assert!(result.sql.contains("UPDATE users SET"));
        assert_eq!(result.key_column.as_deref(), Some("id"));
    }
}
