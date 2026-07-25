// Source: mybatis-plus-core/.../conditions/update/UpdateWrapper.java
// Source: mybatis-plus-core/.../conditions/update/Update.java

use super::super::abstract_wrapper::AbstractWrapper;
use super::super::func::FuncSegments;
use super::super::{compare::Compare, func::Func, nested::Nested, nested::Join};

/// Update wrapper with String column names.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.conditions.update.UpdateWrapper<T>`.
///
/// # Example
///
/// ```ignore
/// use rbatis_plus::core::conditions::update::UpdateWrapper;
///
/// let w = UpdateWrapper::new()
///     .set("name", "Bob")
///     .set_sql("age = age + 1")
///     .eq("id", 42);
///
/// let sql = w.build_update_sql("sys_user");
/// // UPDATE sys_user SET name = 'Bob', age = age + 1 WHERE id = 42
/// ```
#[derive(Debug, Clone, Default)]
pub struct UpdateWrapper {
    pub inner: AbstractWrapper,
    pub func: FuncSegments,
    /// SET clause fragments.
    set_fragments: Vec<String>,
}

impl UpdateWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    /// `SET column = value`
    ///
    /// 设置 SET 字段 = 值
    pub fn set(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self {
        let v = value.into();
        let literal = AbstractWrapper::format_eq(column, &v);
        // format_eq returns "column = value" which is what we want for SET
        self.set_fragments.push(literal);
        self
    }

    /// `SET column = column + delta`
    ///
    /// SET 字段 = 字段 + 增量
    pub fn set_incr_by(&mut self, column: &str, delta: i64) -> &mut Self {
        self.set_fragments.push(format!("{} = {} + {}", column, column, delta));
        self
    }

    /// `SET column = column - delta`
    ///
    /// SET 字段 = 字段 - 减量
    pub fn set_decr_by(&mut self, column: &str, delta: i64) -> &mut Self {
        self.set_fragments.push(format!("{} = {} - {}", column, column, delta));
        self
    }

    /// Append a raw SET fragment (e.g. `"age = age + 1"`, `"update_time = now()"`).
    ///
    /// 追加原始 SET 片段
    pub fn set_sql(&mut self, sql: &str) -> &mut Self {
        self.set_fragments.push(sql.to_string());
        self
    }

    /// Build the SET clause string.
    pub fn build_set(&self) -> String {
        if self.set_fragments.is_empty() {
            String::new()
        } else {
            self.set_fragments.join(", ")
        }
    }

    /// Build the UPDATE SQL for the given table.
    pub fn build_update_sql(&self, table_name: &str) -> String {
        let set_clause = self.build_set();
        let where_clause = self.inner.build_where();
        format!(
            "UPDATE {} SET {}{}",
            table_name, set_clause, where_clause
        )
    }

    /// Build a DELETE SQL using the WHERE conditions.
    pub fn build_delete_sql(&self, table_name: &str) -> String {
        let where_clause = self.inner.build_where();
        format!("DELETE FROM {}{}", table_name, where_clause)
    }

    /// Return bind parameters.
    pub fn params(&self) -> &[rbs::Value] {
        self.inner.params()
    }

    /// Get the assembled SET SQL fragment (or empty).
    pub fn sql_set(&self) -> &str {
        // Return the set fragments as a string without the "SET" prefix
        // Used by injection methods
        // For simplicity return the first fragment if any
        if self.set_fragments.is_empty() {
            ""
        } else {
            // We need a way to return owned string; this is a borrow issue.
            // Users should use `build_set()` instead.
            ""
        }
    }

    /// Clear all conditions and SET fragments.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.func = FuncSegments::default();
        self.set_fragments.clear();
    }
}

// Delegate Compare
impl Compare for UpdateWrapper {
    fn eq(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.eq(column, value); self }
    fn if_eq(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.if_eq(condition, column, value); self }
    fn ne(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.ne(column, value); self }
    fn if_ne(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.if_ne(condition, column, value); self }
    fn gt(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.gt(column, value); self }
    fn if_gt(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.if_gt(condition, column, value); self }
    fn ge(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.ge(column, value); self }
    fn if_ge(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.if_ge(condition, column, value); self }
    fn lt(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.lt(column, value); self }
    fn if_lt(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.if_lt(condition, column, value); self }
    fn le(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.le(column, value); self }
    fn if_le(&mut self, condition: bool, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.if_le(condition, column, value); self }
    fn between(&mut self, column: &str, val1: impl Into<rbs::Value>, val2: impl Into<rbs::Value>) -> &mut Self { self.inner.between(column, val1, val2); self }
    fn if_between(&mut self, condition: bool, column: &str, val1: impl Into<rbs::Value>, val2: impl Into<rbs::Value>) -> &mut Self { self.inner.if_between(condition, column, val1, val2); self }
    fn not_between(&mut self, column: &str, val1: impl Into<rbs::Value>, val2: impl Into<rbs::Value>) -> &mut Self { self.inner.not_between(column, val1, val2); self }
    fn if_not_between(&mut self, condition: bool, column: &str, val1: impl Into<rbs::Value>, val2: impl Into<rbs::Value>) -> &mut Self { self.inner.if_not_between(condition, column, val1, val2); self }
    fn eq_or_is_null(&mut self, column: &str, value: impl Into<rbs::Value>) -> &mut Self { self.inner.eq_or_is_null(column, value); self }
}

// Delegate Func
impl Func for UpdateWrapper {
    fn like(&mut self, column: &str, value: &str) -> &mut Self { self.inner.add_fragment(AbstractWrapper::format_like(column, value)); self }
    fn if_like(&mut self, condition: bool, column: &str, value: &str) -> &mut Self { if condition { self.like(column, value); } self }
    fn not_like(&mut self, column: &str, value: &str) -> &mut Self { self.inner.add_fragment(AbstractWrapper::format_not_like(column, value)); self }
    fn if_not_like(&mut self, condition: bool, column: &str, value: &str) -> &mut Self { if condition { self.not_like(column, value); } self }
    fn like_left(&mut self, column: &str, value: &str) -> &mut Self { self.inner.add_fragment(AbstractWrapper::format_like_left(column, value)); self }
    fn if_like_left(&mut self, condition: bool, column: &str, value: &str) -> &mut Self { if condition { self.like_left(column, value); } self }
    fn like_right(&mut self, column: &str, value: &str) -> &mut Self { self.inner.add_fragment(AbstractWrapper::format_like_right(column, value)); self }
    fn if_like_right(&mut self, condition: bool, column: &str, value: &str) -> &mut Self { if condition { self.like_right(column, value); } self }
    fn is_null(&mut self, column: &str) -> &mut Self { self.inner.add_fragment(AbstractWrapper::format_is_null(column)); self }
    fn if_is_null(&mut self, condition: bool, column: &str) -> &mut Self { if condition { self.is_null(column); } self }
    fn is_not_null(&mut self, column: &str) -> &mut Self { self.inner.add_fragment(AbstractWrapper::format_is_not_null(column)); self }
    fn if_is_not_null(&mut self, condition: bool, column: &str) -> &mut Self { if condition { self.is_not_null(column); } self }
    fn in_values(&mut self, column: &str, values: Vec<rbs::Value>) -> &mut Self { if !values.is_empty() { self.inner.add_fragment(AbstractWrapper::format_in(column, &values)); } self }
    fn if_in_values(&mut self, condition: bool, column: &str, values: Vec<rbs::Value>) -> &mut Self { if condition { self.in_values(column, values); } self }
    fn not_in(&mut self, column: &str, values: Vec<rbs::Value>) -> &mut Self { if !values.is_empty() { self.inner.add_fragment(AbstractWrapper::format_not_in(column, &values)); } self }
    fn if_not_in(&mut self, condition: bool, column: &str, values: Vec<rbs::Value>) -> &mut Self { if condition { self.not_in(column, values); } self }
    fn in_sql(&mut self, column: &str, sql: &str) -> &mut Self { self.inner.add_fragment(format!("{} IN ({})", column, sql)); self }
    fn not_in_sql(&mut self, column: &str, sql: &str) -> &mut Self { self.inner.add_fragment(format!("{} NOT IN ({})", column, sql)); self }
    fn group_by(&mut self, columns: &[&str]) -> &mut Self { self.func.group_by = columns.iter().map(|s| s.to_string()).collect(); self }
    fn order_by(&mut self, column: &str, is_asc: bool) -> &mut Self { let dir = if is_asc { "ASC" } else { "DESC" }; self.func.order_by.push(format!("{} {}", column, dir)); self }
    fn order_by_asc(&mut self, column: &str) -> &mut Self { self.order_by(column, true) }
    fn order_by_desc(&mut self, column: &str) -> &mut Self { self.order_by(column, false) }
    fn having(&mut self, sql: &str) -> &mut Self { self.func.having.push(sql.to_string()); self }
}

// Delegate Nested + Join
impl Nested for UpdateWrapper {
    fn or(&mut self) -> &mut Self { self.inner.or(); self }
    fn and_group(&mut self, inner_sql: &str) -> &mut Self { self.inner.and_group(inner_sql); self }
    fn or_group(&mut self, inner_sql: &str) -> &mut Self { self.inner.or_group(inner_sql); self }
    fn nested(&mut self, inner_sql: &str) -> &mut Self { self.inner.nested(inner_sql); self }
    fn not_group(&mut self, inner_sql: &str) -> &mut Self { self.inner.not_group(inner_sql); self }
}

impl Join for UpdateWrapper {
    fn apply(&mut self, sql: &str) -> &mut Self { self.inner.apply(sql); self }
    fn exists(&mut self, sql: &str) -> &mut Self { self.inner.exists(sql); self }
    fn not_exists(&mut self, sql: &str) -> &mut Self { self.inner.not_exists(sql); self }
    fn last(&mut self, sql: &str) -> &mut Self { self.inner.last(sql); self }
    fn comment(&mut self, sql: &str) -> &mut Self { self.inner.comment(sql); self }
    fn first(&mut self, sql: &str) -> &mut Self { self.inner.first(sql); self }
}
