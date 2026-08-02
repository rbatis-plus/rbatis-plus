//! INSERT IGNORE 模块（对标 mybatis-plus-enhance `insert_ignore` 包）。
//!
//! 提供 INSERT IGNORE 能力，插入时如果主键/唯一键冲突则忽略。

/// INSERT IGNORE 处理器 trait（对标 Java `InsertIgnoreHandler`）。
pub trait InsertIgnoreHandler: Send + Sync + 'static {
    /// 将 INSERT SQL 改写为 INSERT IGNORE SQL。
    ///
    /// 对应 Java `InsertIgnoreHandler.rewrite(String sql)`
    fn rewrite(&self, sql: &str) -> String;

    /// 是否启用 INSERT IGNORE（默认 true）。
    fn enabled(&self) -> bool { true }
}

/// MySQL INSERT IGNORE 处理器。
///
/// 将 `INSERT INTO ...` 改写为 `INSERT IGNORE INTO ...`
#[derive(Debug, Clone)]
pub struct MysqlInsertIgnoreHandler;

impl InsertIgnoreHandler for MysqlInsertIgnoreHandler {
    /// MySQL: `INSERT INTO` → `INSERT IGNORE INTO`
    fn rewrite(&self, sql: &str) -> String {
        let upper = sql.trim_start().to_uppercase();
        // 已经是 INSERT IGNORE，不重复添加
        if upper.starts_with("INSERT IGNORE") || upper.starts_with("INSERT IGNORE INTO") {
            return sql.to_string();
        }
        if upper.starts_with("INSERT INTO") {
            sql.replacen("INSERT INTO", "INSERT IGNORE INTO", 1)
        } else if upper.starts_with("INSERT") {
            sql.replacen("INSERT", "INSERT IGNORE", 1)
        } else {
            sql.to_string()
        }
    }
}

/// PostgreSQL INSERT IGNORE 处理器（使用 ON CONFLICT DO NOTHING）。
///
/// 将 `INSERT INTO ...` 改写为 `INSERT INTO ... ON CONFLICT DO NOTHING`
#[derive(Debug, Clone)]
pub struct PostgreSqlInsertIgnoreHandler;

impl InsertIgnoreHandler for PostgreSqlInsertIgnoreHandler {
    /// PostgreSQL: `INSERT INTO ...` → `INSERT INTO ... ON CONFLICT DO NOTHING`
    fn rewrite(&self, sql: &str) -> String {
        let trimmed = sql.trim_end();
        if trimmed.to_uppercase().starts_with("INSERT") {
            format!("{} ON CONFLICT DO NOTHING", trimmed)
        } else {
            sql.to_string()
        }
    }
}
