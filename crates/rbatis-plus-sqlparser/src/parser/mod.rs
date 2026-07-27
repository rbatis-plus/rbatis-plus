//! SQL 解析器（对标 mybatis-plus-jsqlparser `JSqlParser`）。
//!
//! 提供基础的 SQL 语句类型识别和语句分析。
//! mod.rs 只做模块声明与 re-export（禁止定义类型）。

mod statement_type;
mod parsed_sql;
mod sql_parser;

pub use statement_type::StatementType;
pub use parsed_sql::ParsedSql;
pub use sql_parser::SqlParser;
