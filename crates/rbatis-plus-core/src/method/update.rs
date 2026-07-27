//! UPDATE 方法（`UPDATE <table> SET <set>`）。
use super::{AbstractMethod, MethodResult};
use crate::derive::FieldStrategy;
use crate::metadata::TableInfo;

/// 按 Wrapper 条件更新（`UPDATE <table> SET <column> = ? ...`）。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.Update`
#[derive(Debug)]
pub struct Update;

impl AbstractMethod for Update {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let set_clauses: Vec<String> = table_info.field_list.iter()
            .filter(|f| f.insert_strategy != FieldStrategy::Never)
            .map(|f| format!("{} = ?", f.column))
            .collect();
        let set_sql = set_clauses.join(", ");
        MethodResult {
            sql: format!("UPDATE {} SET {}", table_info.table_name, set_sql),
            method_name: "update".into(),
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
    fn update_sql() {
        let result = Update.generate_sql(&user_table_info());
        assert!(result.sql.contains("UPDATE users SET"));
        assert!(result.sql.contains("name = ?"));
        assert!(!result.sql.contains("big_blob"));
    }
}
