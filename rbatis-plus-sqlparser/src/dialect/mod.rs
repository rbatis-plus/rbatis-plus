//! 数据库方言（对标 mybatis-plus-jsqlparser `IDialect` + `DbType`）。
//!
//! 不同数据库的分页语法不同：
//! - MySQL: `LIMIT offset, size`
//! - PostgreSQL: `LIMIT size OFFSET offset`
//! - Oracle: `SELECT * FROM (SELECT t.*, ROWNUM rn FROM (...) t WHERE ROWNUM <= end) WHERE rn > start`
//! - SQL Server: `SELECT TOP size * FROM (SELECT *, ROW_NUMBER() OVER(...) AS rn FROM ...) WHERE rn > offset`

mod sql_dialect;
pub mod mysql;
pub mod postgresql;
pub mod sqlite;

// Re-exports: 一个文件一个对象，mod.rs 不定义类型
pub use sql_dialect::{SqlDialect, default_dialects};
