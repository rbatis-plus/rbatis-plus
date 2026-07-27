//! SELECT Map（`SELECT <columns> FROM <table> <where>`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按 Wrapper 条件查询，返回 `List<Map<String, Object>>`。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.SelectMaps`
#[derive(Debug)]
pub struct SelectMaps;

impl AbstractMethod for SelectMaps {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let columns = table_info.all_sql_select();
        MethodResult {
            sql: format!("SELECT {} FROM {}", columns, table_info.table_name),
            method_name: "selectMaps".into(),
            key_column: None,
            key_property: None,
        }
    }
}
