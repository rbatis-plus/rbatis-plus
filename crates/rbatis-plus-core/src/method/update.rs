//! 按 Wrapper 条件更新（`UPDATE <table> SET <set> <where>`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按条件更新（SET 由调用方提供）。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.Update`
#[derive(Debug)]
pub struct Update;

impl AbstractMethod for Update {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        MethodResult {
            sql: format!("UPDATE {} SET", table_info.table_name),
            method_name: "update".into(),
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
    fn update_sql() {
        let result = Update.generate_sql(&user_table_info());
        assert!(result.sql.contains("UPDATE users SET"));
    }
}
