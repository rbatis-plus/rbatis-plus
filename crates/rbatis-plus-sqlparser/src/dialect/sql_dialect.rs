//! SQL 方言 trait（对标 Java `IDialect`，mybatis-plus-jsqlparser-support）。
//!
//! 每种数据库实现自己的分页 SQL 改写逻辑。
//!
//! 对应 Java：
//! - `com.baomidou.mybatisplus.extension.plugins.pagination.dialect.IDialect`（`mybatis-plus-jsqlparser-support/.../dialects/IDialect.java`）

use super::{mysql, postgresql, sqlite};

/// SQL 方言 trait（对标 Java `IDialect`）。
///
/// 每种数据库实现自己的分页 SQL 改写逻辑。
///
/// ```rust
/// use rbatis_plus_sqlparser::dialect::SqlDialect;
/// use rbatis_plus_sqlparser::dialect::mysql::MysqlDialect;
///
/// let dialect = MysqlDialect;
/// let sql = dialect.build_pagination_sql("SELECT * FROM users", 0, 10);
/// assert!(sql.contains("LIMIT"));
/// ```
pub trait SqlDialect: Send + Sync {
    /// 方言名称。
    fn name(&self) -> &str;

    /// 改写 SQL 添加分页（对标 `IDialect.buildPaginationSql()`）。
    ///
    /// # 参数
    /// - `sql`: 原始 SQL
    /// - `offset`: 偏移量
    /// - `size`: 每页大小
    fn build_pagination_sql(&self, sql: &str, offset: u64, size: u64) -> String;

    /// 是否支持该方言的 SQL（用于自动检测）。
    fn supports(&self, db_type: &str) -> bool;
}

/// 获取默认方言列表（MySQL / PostgreSQL / SQLite）。
pub fn default_dialects() -> Vec<Box<dyn SqlDialect>> {
    vec![
        Box::new(mysql::MysqlDialect),
        Box::new(postgresql::PostgreSqlDialect),
        Box::new(sqlite::SqliteDialect),
    ]
}
