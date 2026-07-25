//! SQLite 方言（对标 mybatis-plus `SQLiteDialect`）。
//!
//! 分页语法: `LIMIT size OFFSET offset`（与 PostgreSQL 相同）

use super::SqlDialect;

/// SQLite 分页方言。
#[derive(Debug)]
pub struct SqliteDialect;

impl SqlDialect for SqliteDialect {
    fn name(&self) -> &str {
        "SQLite"
    }

    /// SQLite 分页: `SELECT ... LIMIT size OFFSET offset`
    fn build_pagination_sql(&self, sql: &str, offset: u64, size: u64) -> String {
        format!("{} LIMIT {} OFFSET {}", sql.trim_end(), size, offset)
    }

    fn supports(&self, db_type: &str) -> bool {
        db_type.to_lowercase().contains("sqlite")
    }
}
