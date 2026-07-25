//! SQL 解析与改写（对标 mybatis-plus-jsqlparser）。
//!
//! 提供 SQL 解析、方言分页改写、COUNT 改写等能力。
//! 主要被 `PaginationInnerInterceptor` 使用。
//!
//! # 核心模块
//!
//! - [`dialect`] — 数据库方言（MySQL/PostgreSQL/SQLite/Oracle/SQL Server）
//! - [`parser`] — SQL 解析器（语句类型识别、COUNT 改写）
//! - [`rewrite`] — SQL 改写器（分页改写、安全检查）

pub mod dialect;
pub mod parser;
pub mod rewrite;

pub use dialect::{SqlDialect, default_dialects};
pub use dialect::mysql::MysqlDialect;
pub use dialect::postgresql::PostgreSqlDialect;
pub use dialect::sqlite::SqliteDialect;
pub use parser::{ParsedSql, SqlParser, StatementType};
pub use rewrite::SqlRewriter;
