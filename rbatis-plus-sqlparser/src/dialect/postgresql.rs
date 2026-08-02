//! PostgreSQL 方言（对标 mybatis-plus `PostgreSqlDialect`）。
//!
//! 分页语法: `LIMIT size OFFSET offset`

use super::SqlDialect;

/// PostgreSQL 分页方言。
///
/// 对应 Java `com.baomidou.mybatisplus.extension.plugins.pagination.dialects.PostgreSqlDialect`
#[derive(Debug)]
pub struct PostgreSqlDialect;

impl SqlDialect for PostgreSqlDialect {
    fn name(&self) -> &str {
        "PostgreSQL"
    }

    /// PostgreSQL 分页: `SELECT ... LIMIT size OFFSET offset`
    fn build_pagination_sql(&self, sql: &str, offset: u64, size: u64) -> String {
        format!("{} LIMIT {} OFFSET {}", sql.trim_end(), size, offset)
    }

    fn supports(&self, db_type: &str) -> bool {
        let lower = db_type.to_lowercase();
        lower.contains("postgres") || lower.contains("postgresql")
    }
}
