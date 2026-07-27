//! 按 ID 集合查询（`SELECT <columns> FROM <table> WHERE <pk> IN (...)`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按 ID 集合批量查询。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.SelectByIds`
#[derive(Debug)]
pub struct SelectByIds;

impl AbstractMethod for SelectByIds {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let columns = table_info.all_sql_select();
        MethodResult {
            sql: format!("SELECT {} FROM {} WHERE {} IN", columns, table_info.table_name, table_info.key_column),
            method_name: "selectByIds".into(),
            key_column: Some(table_info.key_column.clone()),
            key_property: Some(table_info.key_property.clone()),
        }
    }
}
