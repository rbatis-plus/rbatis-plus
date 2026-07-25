//! 数据库方言（对标 mybatis-plus-jsqlparser `IDialect` + `DbType`）。
//!
//! 不同数据库的分页语法不同：
//! - MySQL: `LIMIT offset, size`
//! - PostgreSQL: `LIMIT size OFFSET offset`
//! - Oracle: `SELECT * FROM (SELECT t.*, ROWNUM rn FROM (...) t WHERE ROWNUM <= end) WHERE rn > start`
//! - SQL Server: `SELECT TOP size * FROM (SELECT *, ROW_NUMBER() OVER(...) AS rn FROM ...) WHERE rn > offset`

pub mod mysql;
pub mod postgresql;
pub mod sqlite;

/// SQL 方言 trait（对标 Java `IDialect`）。
///
/// 每种数据库实现自己的分页 SQL 改写逻辑。
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

/// 获取默认方言列表。
pub fn default_dialects() -> Vec<Box<dyn SqlDialect>> {
    vec![
        Box::new(mysql::MysqlDialect),
        Box::new(postgresql::PostgreSqlDialect),
        Box::new(sqlite::SqliteDialect),
    ]
}
