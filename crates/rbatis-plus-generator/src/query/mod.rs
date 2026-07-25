//! 数据库元数据查询模块（对标 mybatis-plus-generator `query` 包）。

pub mod table_info;

pub use table_info::{TableField, TableInfo};
