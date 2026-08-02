// Source: mybatis-plus-core/.../conditions/interfaces/Func.java

use rbs::Value;

/// Function condition methods: LIKE, IN, IS NULL, GROUP BY, ORDER BY, HAVING.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.conditions.interfaces.Func<R>`.
pub trait Func {
    /// `column LIKE '%value%'`
    ///
    /// LIKE '%值%'
    fn like(&mut self, column: &str, value: &str) -> &mut Self;
    fn if_like(&mut self, condition: bool, column: &str, value: &str) -> &mut Self;

    /// `column NOT LIKE '%value%'`
    ///
    /// NOT LIKE '%值%'
    fn not_like(&mut self, column: &str, value: &str) -> &mut Self;
    fn if_not_like(&mut self, condition: bool, column: &str, value: &str) -> &mut Self;

    /// `column LIKE '%value'`
    ///
    /// LIKE '%值'
    fn like_left(&mut self, column: &str, value: &str) -> &mut Self;
    fn if_like_left(&mut self, condition: bool, column: &str, value: &str) -> &mut Self;

    /// `column LIKE 'value%'`
    ///
    /// LIKE '值%'
    fn like_right(&mut self, column: &str, value: &str) -> &mut Self;
    fn if_like_right(&mut self, condition: bool, column: &str, value: &str) -> &mut Self;

    /// `column IS NULL`
    ///
    /// 字段 IS NULL
    fn is_null(&mut self, column: &str) -> &mut Self;
    fn if_is_null(&mut self, condition: bool, column: &str) -> &mut Self;

    /// `column IS NOT NULL`
    ///
    /// 字段 IS NOT NULL
    fn is_not_null(&mut self, column: &str) -> &mut Self;
    fn if_is_not_null(&mut self, condition: bool, column: &str) -> &mut Self;

    /// `column IN (v1, v2, ...)`
    ///
    /// IN (值1, 值2, ...)
    fn in_values(&mut self, column: &str, values: Vec<Value>) -> &mut Self;
    fn if_in_values(&mut self, condition: bool, column: &str, values: Vec<Value>) -> &mut Self;

    /// `column NOT IN (v1, v2, ...)`
    ///
    /// NOT IN (值1, 值2, ...)
    fn not_in(&mut self, column: &str, values: Vec<Value>) -> &mut Self;
    fn if_not_in(&mut self, condition: bool, column: &str, values: Vec<Value>) -> &mut Self;

    /// `column IN (subquery_sql)`
    ///
    /// IN (子查询)
    fn in_sql(&mut self, column: &str, sql: &str) -> &mut Self;

    /// `column NOT IN (subquery_sql)`
    ///
    /// NOT IN (子查询)
    fn not_in_sql(&mut self, column: &str, sql: &str) -> &mut Self;

    /// `GROUP BY col1, col2, ...`
    ///
    /// 分组：GROUP BY 字段, ...
    fn group_by(&mut self, columns: &[&str]) -> &mut Self;

    /// `ORDER BY col ASC/DESC`
    ///
    /// 排序：ORDER BY 字段 ASC/DESC
    fn order_by(&mut self, column: &str, is_asc: bool) -> &mut Self;
    fn order_by_asc(&mut self, column: &str) -> &mut Self;
    fn order_by_desc(&mut self, column: &str) -> &mut Self;

    /// `HAVING sqlFragment`
    ///
    /// HAVING (sql 片段)
    fn having(&mut self, sql: &str) -> &mut Self;
}

// ── Helper: Func also needs ORDER BY / GROUP BY / HAVING storage ───────

/// Extra segments that are not WHERE conditions but part of the SQL.
#[derive(Debug, Clone, Default)]
pub struct FuncSegments {
    pub group_by: Vec<String>,
    pub order_by: Vec<String>,
    pub having: Vec<String>,
}

impl FuncSegments {
    pub fn build_group_by(&self) -> String {
        if self.group_by.is_empty() {
            String::new()
        } else {
            format!(" GROUP BY {}", self.group_by.join(", "))
        }
    }

    pub fn build_order_by(&self) -> String {
        if self.order_by.is_empty() {
            String::new()
        } else {
            format!(" ORDER BY {}", self.order_by.join(", "))
        }
    }

    pub fn build_having(&self) -> String {
        if self.having.is_empty() {
            String::new()
        } else {
            format!(" HAVING {}", self.having.join(" AND "))
        }
    }
}
