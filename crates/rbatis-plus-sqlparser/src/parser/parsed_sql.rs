//! SQL 解析结果结构体。
//!
//! 对应 Java：`com.baomidou.mybatisplus.extension.parser.IJSqlParser` 的 `Statement` 接口解析产物。
//! mybatis-plus-jsqlparser 中，ParsedSql 存储原始 SQL 与各种语句特征标记。
//!
//! 文件来源参考：`mybatis-plus-jsqlparser-support/.../plugins/inner/PaginationInnerInterceptor.java`

use super::statement_type::StatementType;

/// SQL 解析结果（对标 Java `Statement` 的简化版）。
#[derive(Debug, Clone)]
pub struct ParsedSql {
    /// 语句类型。
    pub statement_type: StatementType,
    /// 原始 SQL。
    pub original_sql: String,
    /// 去除前后空白和注释后的 SQL。
    pub trimmed_sql: String,
    /// 是否包含 DISTINCT。
    pub has_distinct: bool,
    /// 是否包含 GROUP BY。
    pub has_group_by: bool,
    /// 是否包含 ORDER BY。
    pub has_order_by: bool,
    /// 是否包含 UNION。
    pub has_union: bool,
    /// 是否包含 FOR UPDATE / FOR SHARE。
    pub has_for_update: bool,
    /// 是否包含 LIMIT 子句（已有的）。
    pub has_limit: bool,
}
