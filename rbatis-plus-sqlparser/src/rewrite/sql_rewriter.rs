//! SQL 改写器（对标 mybatis-plus-jsqlparser `AbstractJsqlParser` + 分页改写）。
//!
//! 负责将原始 SQL 改写为带分页的 SQL。
//!
//! 对应 Java：
//! - `com.baomidou.mybatisplus.extension.parser.JsqlParserSupport`
//! - `com.baomidou.mybatisplus.extension.plugins.inner.PaginationInnerInterceptor`
//!
//! 文件来源参考：`mybatis-plus-jsqlparser-support/.../plugins/inner/PaginationInnerInterceptor.java`

use crate::dialect::SqlDialect;
use crate::parser::SqlParser;

/// SQL 改写器（对标 Java `AbstractJsqlParser` 的分页改写功能）。
///
/// 负责将原始 SQL 改写为带分页的 SQL。
pub struct SqlRewriter;

impl SqlRewriter {
    /// 改写 SQL 添加分页（对标 `JsqlParserSupport.parserSingle()` + 方言改写）。
    ///
    /// # 参数
    /// - `sql`: 原始 SQL
    /// - `page_no`: 页码（从 1 开始）
    /// - `page_size`: 每页大小
    /// - `dialect`: 数据库方言
    ///
    /// # 返回
    /// 改写后的分页 SQL。
    pub fn rewrite_pagination(
        sql: &str,
        page_no: u64,
        page_size: u64,
        dialect: &dyn SqlDialect,
    ) -> String {
        let offset = (page_no - 1) * page_size;
        dialect.build_pagination_sql(sql, offset, page_size)
    }

    /// 改写 COUNT SQL（对标 `JsqlParserSupport.parserMulti()` + COUNT 改写）。
    ///
    /// 将原始查询 SQL 改写为 COUNT 查询。
    pub fn rewrite_count(sql: &str) -> String {
        SqlParser::get_count_sql(sql)
    }

    /// 检查 SQL 是否可以安全分页（不包含 FOR UPDATE 等不安全语句）。
    ///
    /// 对应 Java `PaginationInnerInterceptor.consumes()` 中的安全检查
    pub fn can_paginate(sql: &str) -> bool {
        let parsed = SqlParser::parse(sql);
        // FOR UPDATE / FOR SHARE 不能分页
        if parsed.has_for_update {
            return false;
        }
        // 只有 SELECT 可以分页
        matches!(parsed.statement_type, crate::parser::StatementType::Select)
    }
}
