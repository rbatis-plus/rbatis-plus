// Source: mybatis-plus-core/.../conditions/query/QueryWrapper.java
// Source: mybatis-plus-core/.../conditions/query/Query.java

use super::super::abstract_wrapper::AbstractWrapper;
use super::super::func::FuncSegments;
use super::super::{compare::Compare, func::Func, nested::Nested, join::Join};

/// Query wrapper with String column names.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.conditions.query.QueryWrapper<T>`.
///
/// # Example
///
/// ```ignore
/// use rbatis_plus::core::conditions::query::QueryWrapper;
///
/// let w = QueryWrapper::new()
///     .eq("name", "Alice")
///     .ge("age", 18)
///     .like("email", "gmail")
///     .order_by_desc("create_time");
///
/// let sql = w.build_select_sql("sys_user");
/// // SELECT * FROM sys_user WHERE name = 'Alice' AND age >= 18 AND email LIKE '%gmail%' ORDER BY create_time DESC
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryWrapper {
    pub inner: AbstractWrapper,
    pub func: FuncSegments,
    /// Selected columns (empty = `*`).
    select_columns: Vec<String>,
}

impl QueryWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the SELECT columns. Overwrites previous selection.
    ///
    /// 设置查询字段
    pub fn select(&mut self, columns: &[&str]) -> &mut Self {
        self.select_columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the SELECT SQL for the given table.
    pub fn build_select_sql(&self, table_name: &str) -> String {
        let cols = if self.select_columns.is_empty() {
            "*".to_string()
        } else {
            self.select_columns.join(", ")
        };

        let mut sql = String::new();

        // sql_first（最前置，如 hint）
        if !self.inner.sql_first.is_empty() {
            sql.push_str(&self.inner.sql_first);
            sql.push(' ');
        }

        // sql_comment（前置注释）
        if !self.inner.sql_comment.is_empty() {
            sql.push_str(&self.inner.sql_comment);
            sql.push(' ');
        }

        sql.push_str(&format!("SELECT {} FROM {}", cols, table_name));
        let where_clause = self.inner.build_where();
        sql.push_str(&where_clause);
        sql.push_str(&self.func.build_group_by());
        sql.push_str(&self.func.build_having());
        sql.push_str(&self.func.build_order_by());
        if !self.inner.sql_last.is_empty() {
            sql.push(' ');
            sql.push_str(&self.inner.sql_last);
        }
        sql
    }

    /// Build a COUNT SQL for the given table (for pagination).
    pub fn build_count_sql(&self, table_name: &str) -> String {
        let where_clause = self.inner.build_where();
        let mut sql = format!("SELECT COUNT(*) AS total FROM {}{}", table_name, where_clause);
        if !self.func.build_group_by().is_empty() {
            sql.push_str(&self.func.build_group_by());
            sql.push_str(&self.func.build_having());
            // Wrap in subquery if GROUP BY is present
            sql = format!("SELECT COUNT(*) AS total FROM ({}) t", sql);
        }
        sql
    }

    /// Return bind parameters.
    pub fn params(&self) -> &[rbs::Value] {
        self.inner.params()
    }

    /// Clear all conditions.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.func = FuncSegments::default();
        self.select_columns.clear();
    }
}

// Delegate Compare to AbstractWrapper
impl Compare for QueryWrapper {
    fn eq(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.eq(column, value);
        self
    }
    fn if_eq(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.if_eq(condition, column, value);
        self
    }
    fn ne(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.ne(column, value);
        self
    }
    fn if_ne(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.if_ne(condition, column, value);
        self
    }
    fn gt(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.gt(column, value);
        self
    }
    fn if_gt(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.if_gt(condition, column, value);
        self
    }
    fn ge(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.ge(column, value);
        self
    }
    fn if_ge(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.if_ge(condition, column, value);
        self
    }
    fn lt(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.lt(column, value);
        self
    }
    fn if_lt(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.if_lt(condition, column, value);
        self
    }
    fn le(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.le(column, value);
        self
    }
    fn if_le(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.if_le(condition, column, value);
        self
    }
    fn between(
        &mut self,
        column: &str,
        val1: impl Into<rbs::Value>,
        val2: impl Into<rbs::Value>,
    ) -> &mut Self {
        self.inner.between(column, val1, val2);
        self
    }
    fn if_between(
        &mut self,
        condition: bool,
        column: &str,
        val1: impl Into<rbs::Value>,
        val2: impl Into<rbs::Value>,
    ) -> &mut Self {
        self.inner.if_between(condition, column, val1, val2);
        self
    }
    fn not_between(
        &mut self,
        column: &str,
        val1: impl Into<rbs::Value>,
        val2: impl Into<rbs::Value>,
    ) -> &mut Self {
        self.inner.not_between(column, val1, val2);
        self
    }
    fn if_not_between(
        &mut self,
        condition: bool,
        column: &str,
        val1: impl Into<rbs::Value>,
        val2: impl Into<rbs::Value>,
    ) -> &mut Self {
        self.inner.if_not_between(condition, column, val1, val2);
        self
    }
    fn eq_or_is_null(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        self.inner.eq_or_is_null(column, value);
        self
    }
}

// Delegate Func to AbstractWrapper + FuncSegments
impl Func for QueryWrapper {
    fn like(&mut self, column: &str, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like(column, value));
        self
    }
    fn if_like(&mut self, condition: bool, column: &str, value: &str) -> &mut Self {
        if condition { self.like(column, value); }
        self
    }
    fn not_like(&mut self, column: &str, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_not_like(column, value));
        self
    }
    fn if_not_like(&mut self, condition: bool, column: &str, value: &str) -> &mut Self {
        if condition { self.not_like(column, value); }
        self
    }
    fn like_left(&mut self, column: &str, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like_left(column, value));
        self
    }
    fn if_like_left(&mut self, condition: bool, column: &str, value: &str) -> &mut Self {
        if condition { self.like_left(column, value); }
        self
    }
    fn like_right(&mut self, column: &str, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like_right(column, value));
        self
    }
    fn if_like_right(&mut self, condition: bool, column: &str, value: &str) -> &mut Self {
        if condition { self.like_right(column, value); }
        self
    }
    fn is_null(&mut self, column: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_is_null(column));
        self
    }
    fn if_is_null(&mut self, condition: bool, column: &str) -> &mut Self {
        if condition { self.is_null(column); }
        self
    }
    fn is_not_null(&mut self, column: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_is_not_null(column));
        self
    }
    fn if_is_not_null(&mut self, condition: bool, column: &str) -> &mut Self {
        if condition { self.is_not_null(column); }
        self
    }
    fn in_values(&mut self, column: &str, values: Vec<rbs::Value>) -> &mut Self {
        if !values.is_empty() {
            self.inner.add_fragment(AbstractWrapper::format_in(column, &values));
        }
        self
    }
    fn if_in_values(&mut self, condition: bool, column: &str, values: Vec<rbs::Value>) -> &mut Self {
        if condition { self.in_values(column, values); }
        self
    }
    fn not_in(&mut self, column: &str, values: Vec<rbs::Value>) -> &mut Self {
        if !values.is_empty() {
            self.inner.add_fragment(AbstractWrapper::format_not_in(column, &values));
        }
        self
    }
    fn if_not_in(&mut self, condition: bool, column: &str, values: Vec<rbs::Value>) -> &mut Self {
        if condition { self.not_in(column, values); }
        self
    }
    fn in_sql(&mut self, column: &str, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("{} IN ({})", column, sql));
        self
    }
    fn not_in_sql(&mut self, column: &str, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("{} NOT IN ({})", column, sql));
        self
    }
    fn group_by(&mut self, columns: &[&str]) -> &mut Self {
        self.func.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }
    fn order_by(&mut self, column: &str, is_asc: bool) -> &mut Self {
        let dir = if is_asc { "ASC" } else { "DESC" };
        self.func.order_by.push(format!("{} {}", column, dir));
        self
    }
    fn order_by_asc(&mut self, column: &str) -> &mut Self {
        self.order_by(column, true)
    }
    fn order_by_desc(&mut self, column: &str) -> &mut Self {
        self.order_by(column, false)
    }
    fn having(&mut self, sql: &str) -> &mut Self {
        self.func.having.push(sql.to_string());
        self
    }
}

// Delegate Nested + Join to AbstractWrapper
impl Nested for QueryWrapper {
    fn or(&mut self) -> &mut Self { self.inner.or(); self }
    fn and_group(&mut self, inner_sql: &str) -> &mut Self { self.inner.and_group(inner_sql); self }
    fn or_group(&mut self, inner_sql: &str) -> &mut Self { self.inner.or_group(inner_sql); self }
    fn nested(&mut self, inner_sql: &str) -> &mut Self { self.inner.nested(inner_sql); self }
    fn not_group(&mut self, inner_sql: &str) -> &mut Self { self.inner.not_group(inner_sql); self }
}

impl Join for QueryWrapper {
    fn apply(&mut self, sql: &str) -> &mut Self { self.inner.apply(sql); self }
    fn exists(&mut self, sql: &str) -> &mut Self { self.inner.exists(sql); self }
    fn not_exists(&mut self, sql: &str) -> &mut Self { self.inner.not_exists(sql); self }
    fn last(&mut self, sql: &str) -> &mut Self { self.inner.last(sql); self }
    fn comment(&mut self, sql: &str) -> &mut Self { self.inner.comment(sql); self }
    fn first(&mut self, sql: &str) -> &mut Self { self.inner.first(sql); self }
}
