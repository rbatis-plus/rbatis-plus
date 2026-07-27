//! SQL 改写器模块（mod.rs 只做声明与 re-export）。
//!
//! 对应 Java `com.baomidou.mybatisplus.extension.parser.JsqlParserSupport`
//! + 分页改写逻辑。

mod sql_rewriter;

pub use sql_rewriter::SqlRewriter;
