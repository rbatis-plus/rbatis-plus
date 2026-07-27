//! 按 ID 集合删除（`DELETE FROM <table> WHERE <pk> IN (...)`）。
use super::{AbstractMethod, MethodResult};
use crate::metadata::TableInfo;

/// 按 ID 集合批量删除。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.injector.methods.DeleteByIds`
/// 注：此方法在 Wrapper 构造后由 Executor 执行，SQL 模板为占位。
#[derive(Debug)]
pub struct DeleteByIds;

impl AbstractMethod for DeleteByIds {
    fn generate_sql(&self, table_info: &TableInfo) -> MethodResult {
        MethodResult {
            sql: format!("DELETE FROM {} WHERE {} IN", table_info.table_name, table_info.key_column),
            method_name: "deleteByIds".into(),
            key_column: Some(table_info.key_column.clone()),
            key_property: Some(table_info.key_property.clone()),
        }
    }
}
