//! SELECT 单行（`SELECT <columns> FROM <table> <where>`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按 Wrapper 条件查询单条记录。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.SelectOne`
#[derive(Debug)]
pub struct SelectOne;

impl AbstractMethod for SelectOne {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        let columns = table_info.all_sql_select();
        MethodResult {
            sql: format!("SELECT {} FROM {}", columns, table_info.table_name),
            method_name: "selectOne".into(),
            key_column: None,
            key_property: None,
        }
    }
}
