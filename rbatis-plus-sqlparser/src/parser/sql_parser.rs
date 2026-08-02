//! SQL 解析器（对标 Java `JSqlParser` + `CCJSqlParserUtil`）。
//!
//! 使用正则表达式进行轻量级 SQL 解析。
//! 生产版本可替换为完整的 SQL parser（如 sqlparser-rs crate）。
//!
//! 对应 Java：`com.baomidou.mybatisplus.extension.parser.IJSqlParser`
//! + `net.sf.jsqlparser.parser.CCJSqlParserUtil.parse()`
//!
//! 文件来源参考：`mybatis-plus-jsqlparser-support/.../plugins/inner/PaginationInnerInterceptor.java`

use regex::Regex;

use super::parsed_sql::ParsedSql;
use super::statement_type::StatementType;

/// SQL 解析器（对标 Java `JSqlParser` + `CCJSqlParserUtil`）。
///
/// 使用正则表达式进行轻量级 SQL 解析。
/// 生产版本可替换为完整的 SQL parser（如 sqlparser-rs crate）。
pub struct SqlParser;

impl SqlParser {
    /// 解析 SQL 语句（对标 `JSqlParser.parse(sql)`）。
    pub fn parse(sql: &str) -> ParsedSql {
        let trimmed = sql.trim().to_string();
        let upper = trimmed.to_uppercase();

        let statement_type = if upper.starts_with("SELECT") {
            StatementType::Select
        } else if upper.starts_with("INSERT") {
            StatementType::Insert
        } else if upper.starts_with("UPDATE") {
            StatementType::Update
        } else if upper.starts_with("DELETE") {
            StatementType::Delete
        } else {
            StatementType::Other(trimmed.clone())
        };

        ParsedSql {
            statement_type,
            original_sql: sql.to_string(),
            trimmed_sql: trimmed,
            has_distinct: upper.contains(" DISTINCT "),
            has_group_by: upper.contains(" GROUP BY "),
            has_order_by: upper.contains(" ORDER BY "),
            has_union: upper.contains(" UNION "),
            has_for_update: upper.contains(" FOR UPDATE") || upper.contains(" FOR SHARE"),
            has_limit: upper.contains(" LIMIT "),
        }
    }

    /// 获取 COUNT SQL（对标 `ISqlParser.getOriginalCountSql()`）。
    ///
    /// 将 `SELECT ... FROM ...` 转为 `SELECT COUNT(*) FROM ...`。
    /// 如果包含 GROUP BY，则包装为子查询。
    pub fn get_count_sql(sql: &str) -> String {
        let parsed = Self::parse(sql);

        if parsed.has_group_by || parsed.has_union || parsed.has_distinct {
            // 复杂查询包装为子查询
            format!("SELECT COUNT(*) AS total FROM ({}) _t", sql.trim_end())
        } else {
            // 简单查询：替换 SELECT ... FROM 为 SELECT COUNT(*) FROM
            Self::simple_count_rewrite(sql)
        }
    }

    /// 简单 COUNT 改写（正则替换 SELECT ... FROM → SELECT COUNT(*) FROM）。
    fn simple_count_rewrite(sql: &str) -> String {
        let re = Regex::new(r"(?is)^\s*SELECT\s+.+?\s+FROM\s+").unwrap();
        if re.find(sql).is_some() {
            re.replace(sql, "SELECT COUNT(*) AS total FROM ").to_string()
        } else {
            // 回退：包装子查询
            format!("SELECT COUNT(*) AS total FROM ({}) _t", sql.trim_end())
        }
    }

    /// 优化 SELECT *（对标 `SelectItemVisitorAdapter` + 列裁剪）。
    ///
    /// 如果 SQL 是 `SELECT *`，可以替换为具体列名。
    pub fn replace_select_star(sql: &str, columns: &[&str]) -> String {
        let re = Regex::new(r"(?is)^\s*SELECT\s+\*\s+FROM\s+").unwrap();
        if re.find(sql).is_some() {
            let cols = columns.join(", ");
            re.replace(sql, &format!("SELECT {} FROM ", cols)).to_string()
        } else {
            sql.to_string()
        }
    }
}
