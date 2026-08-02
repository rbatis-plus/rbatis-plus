//! MySQL 方言（对标 mybatis-plus `MySqlDialect`）。
//!
//! 分页语法: `LIMIT offset, size`

use super::SqlDialect;

/// MySQL 分页方言。
///
/// 对应 Java `com.baomidou.mybatisplus.extension.plugins.pagination.dialects.MySqlDialect`
#[derive(Debug)]
pub struct MysqlDialect;

impl SqlDialect for MysqlDialect {
    fn name(&self) -> &str {
        "MySQL"
    }

    /// MySQL 分页: `SELECT ... LIMIT offset, size`
    fn build_pagination_sql(&self, sql: &str, offset: u64, size: u64) -> String {
        format!("{} LIMIT {}, {}", sql.trim_end(), offset, size)
    }

    fn supports(&self, db_type: &str) -> bool {
        let lower = db_type.to_lowercase();
        lower.contains("mysql") || lower.contains("mariadb")
    }
}
