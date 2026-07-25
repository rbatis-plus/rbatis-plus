// Source: mybatis-plus-core/.../conditions/query/LambdaQueryWrapper.java
// Source: mybatis-plus-core/.../conditions/AbstractLambdaWrapper.java

use super::super::abstract_wrapper::AbstractWrapper;
use super::super::func::FuncSegments;
use super::column::Column;
use rbs::Value;
use std::marker::PhantomData;

/// 基于类型安全列引用的查询构建器（对标 Java `LambdaQueryWrapper<T>`）。
///
/// 使用 `Column<F>` 替代字符串列名，编译期保证列名正确性。
/// 列引用通常由 `#[derive(TableName)]` 宏自动生成。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.core.conditions.query.LambdaQueryWrapper<T>`
/// - `com.baomidou.mybatisplus.core.conditions.AbstractLambdaWrapper<T, Children>`
///
/// # Example
///
/// ```ignore
/// use rbatis_plus::core::conditions::query::LambdaQueryWrapper;
///
/// // 假设 User 已 derive TableName，生成了 User::column_name() 等方法
/// let w = LambdaQueryWrapper::<User>::new()
///     .eq(User::column_name(), "Alice")
///     .ge(User::column_age(), 18)
///     .order_by_desc(User::column_create_time());
///
/// let sql = w.build_select_sql("sys_user");
/// // SELECT * FROM sys_user WHERE name = 'Alice' AND age >= 18 ORDER BY create_time DESC
/// ```
#[derive(Debug, Clone)]
pub struct LambdaQueryWrapper<T> {
    /// 内部条件构建状态。
    pub inner: AbstractWrapper,
    /// GROUP BY / ORDER BY / HAVING 状态。
    pub func: FuncSegments,
    /// SELECT 列列表（空 = `*`）。
    select_columns: Vec<String>,
    /// 实体类型幽灵标记。
    _phantom: PhantomData<T>,
}

impl<T> Default for LambdaQueryWrapper<T> {
    fn default() -> Self {
        Self {
            inner: AbstractWrapper::default(),
            func: FuncSegments::default(),
            select_columns: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T> LambdaQueryWrapper<T> {
    /// 创建空的 Lambda 查询构建器。
    pub fn new() -> Self {
        Self::default()
    }

    // ── SELECT ─────────────────────────────────────────────────────────

    /// 设置查询字段（类型安全列引用）。
    ///
    /// 对应 Java `LambdaQueryWrapper.select(SFunction...)`
    pub fn select<F>(&mut self, column: Column<F>) -> &mut Self {
        self.select_columns.push(column.name().to_string());
        self
    }

    /// 批量设置查询字段。
    pub fn select_columns(&mut self, columns: &[&str]) -> &mut Self {
        self.select_columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    // ── Compare（类型安全列引用版本）────────────────────────────────

    /// `column = value`（等值比较）
    ///
    /// 对应 Java `AbstractWrapper.eq(boolean condition, R column, Object val)`
    pub fn eq<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_eq(column.name(), &v));
        self
    }

    /// 条件等值比较 — 仅当 `condition` 为 true 时生效。
    pub fn if_eq<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.eq(column, value); }
        self
    }

    /// `column <> value`（不等比较）
    ///
    /// 对应 Java `AbstractWrapper.ne(...)`
    pub fn ne<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_ne(column.name(), &v));
        self
    }

    /// 条件不等比较。
    pub fn if_ne<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.ne(column, value); }
        self
    }

    /// `column > value`（大于）
    ///
    /// 对应 Java `AbstractWrapper.gt(...)`
    pub fn gt<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_gt(column.name(), &v));
        self
    }

    /// 条件大于比较。
    pub fn if_gt<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.gt(column, value); }
        self
    }

    /// `column >= value`（大于等于）
    ///
    /// 对应 Java `AbstractWrapper.ge(...)`
    pub fn ge<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_ge(column.name(), &v));
        self
    }

    /// 条件大于等于比较。
    pub fn if_ge<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.ge(column, value); }
        self
    }

    /// `column < value`（小于）
    ///
    /// 对应 Java `AbstractWrapper.lt(...)`
    pub fn lt<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_lt(column.name(), &v));
        self
    }

    /// 条件小于比较。
    pub fn if_lt<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.lt(column, value); }
        self
    }

    /// `column <= value`（小于等于）
    ///
    /// 对应 Java `AbstractWrapper.le(...)`
    pub fn le<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_le(column.name(), &v));
        self
    }

    /// 条件小于等于比较。
    pub fn if_le<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.le(column, value); }
        self
    }

    /// `column BETWEEN val1 AND val2`
    ///
    /// 对应 Java `AbstractWrapper.between(...)`
    pub fn between<F>(&mut self, column: Column<F>, val1: impl Into<Value>, val2: impl Into<Value>) -> &mut Self {
        let v1 = val1.into();
        let v2 = val2.into();
        self.inner.add_fragment(AbstractWrapper::format_between(column.name(), &v1, &v2));
        self
    }

    /// 条件 BETWEEN。
    pub fn if_between<F>(
        &mut self, condition: bool, column: Column<F>,
        val1: impl Into<Value>, val2: impl Into<Value>,
    ) -> &mut Self {
        if condition { self.between(column, val1, val2); }
        self
    }

    /// `column NOT BETWEEN val1 AND val2`
    ///
    /// 对应 Java `AbstractWrapper.notBetween(...)`
    pub fn not_between<F>(&mut self, column: Column<F>, val1: impl Into<Value>, val2: impl Into<Value>) -> &mut Self {
        let v1 = val1.into();
        let v2 = val2.into();
        self.inner.add_fragment(AbstractWrapper::format_not_between(column.name(), &v1, &v2));
        self
    }

    /// 条件 NOT BETWEEN。
    pub fn if_not_between<F>(
        &mut self, condition: bool, column: Column<F>,
        val1: impl Into<Value>, val2: impl Into<Value>,
    ) -> &mut Self {
        if condition { self.not_between(column, val1, val2); }
        self
    }

    /// `column = value OR column IS NULL`
    ///
    /// 对应 Java `AbstractWrapper.eqOrIsNull(...)`
    pub fn eq_or_is_null<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        if matches!(v, Value::Null) {
            self.inner.add_fragment(AbstractWrapper::format_is_null(column.name()));
        } else {
            self.inner.add_fragment(AbstractWrapper::format_eq(column.name(), &v));
        }
        self
    }

    // ── Func（类型安全列引用版本）──────────────────────────────────

    /// `column LIKE '%value%'`（模糊匹配）
    ///
    /// 对应 Java `AbstractWrapper.like(...)`
    pub fn like<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like(column.name(), value));
        self
    }

    /// 条件 LIKE。
    pub fn if_like<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.like(column, value); }
        self
    }

    /// `column NOT LIKE '%value%'`
    ///
    /// 对应 Java `AbstractWrapper.notLike(...)`
    pub fn not_like<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_not_like(column.name(), value));
        self
    }

    /// 条件 NOT LIKE。
    pub fn if_not_like<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.not_like(column, value); }
        self
    }

    /// `column LIKE 'value%'`（右匹配）
    ///
    /// 对应 Java `AbstractWrapper.likeRight(...)`
    pub fn like_right<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like_right(column.name(), value));
        self
    }

    /// 条件右匹配 LIKE。
    pub fn if_like_right<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.like_right(column, value); }
        self
    }

    /// `column LIKE '%value'`（左匹配）
    ///
    /// 对应 Java `AbstractWrapper.likeLeft(...)`
    pub fn like_left<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like_left(column.name(), value));
        self
    }

    /// 条件左匹配 LIKE。
    pub fn if_like_left<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.like_left(column, value); }
        self
    }

    /// `column IS NULL`
    ///
    /// 对应 Java `AbstractWrapper.isNull(...)`
    pub fn is_null<F>(&mut self, column: Column<F>) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_is_null(column.name()));
        self
    }

    /// 条件 IS NULL。
    pub fn if_is_null<F>(&mut self, condition: bool, column: Column<F>) -> &mut Self {
        if condition { self.is_null(column); }
        self
    }

    /// `column IS NOT NULL`
    ///
    /// 对应 Java `AbstractWrapper.isNotNull(...)`
    pub fn is_not_null<F>(&mut self, column: Column<F>) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_is_not_null(column.name()));
        self
    }

    /// 条件 IS NOT NULL。
    pub fn if_is_not_null<F>(&mut self, condition: bool, column: Column<F>) -> &mut Self {
        if condition { self.is_not_null(column); }
        self
    }

    /// `column IN (v1, v2, ...)`
    ///
    /// 对应 Java `AbstractWrapper.in(...)`
    pub fn in_values<F>(&mut self, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if !values.is_empty() {
            self.inner.add_fragment(AbstractWrapper::format_in(column.name(), &values));
        }
        self
    }

    /// 条件 IN。
    pub fn if_in_values<F>(&mut self, condition: bool, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if condition { self.in_values(column, values); }
        self
    }

    /// `column NOT IN (v1, v2, ...)`
    ///
    /// 对应 Java `AbstractWrapper.notIn(...)`
    pub fn not_in<F>(&mut self, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if !values.is_empty() {
            self.inner.add_fragment(AbstractWrapper::format_not_in(column.name(), &values));
        }
        self
    }

    /// 条件 NOT IN。
    pub fn if_not_in<F>(&mut self, condition: bool, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if condition { self.not_in(column, values); }
        self
    }

    /// `column IN (subquery)`
    ///
    /// 对应 Java `AbstractWrapper.inSql(...)`
    pub fn in_sql<F>(&mut self, column: Column<F>, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("{} IN ({})", column.name(), sql));
        self
    }

    /// `column NOT IN (subquery)`
    ///
    /// 对应 Java `AbstractWrapper.notInSql(...)`
    pub fn not_in_sql<F>(&mut self, column: Column<F>, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("{} NOT IN ({})", column.name(), sql));
        self
    }

    // ── GROUP BY / ORDER BY / HAVING ──────────────────────────────────

    /// GROUP BY 多列（类型安全列引用，支持混合字段类型）。
    ///
    /// 对应 Java `AbstractWrapper.groupBy(boolean condition, R... columns)`
    ///
    /// 因为 Rust 泛型不允许 slice 中混合不同类型，此方法接受列名字符串切片。
    /// 推荐配合 derive 宏生成的 `COLUMN_*` 常量使用。
    pub fn group_by(&mut self, columns: &[&str]) -> &mut Self {
        self.func.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// ORDER BY（类型安全列引用）。
    ///
    /// 对应 Java `AbstractWrapper.orderBy(boolean condition, boolean isAsc, R... columns)`
    pub fn order_by<F>(&mut self, column: Column<F>, is_asc: bool) -> &mut Self {
        let dir = if is_asc { "ASC" } else { "DESC" };
        self.func.order_by.push(format!("{} {}", column.name(), dir));
        self
    }

    /// ORDER BY ASC（类型安全列引用）。
    pub fn order_by_asc<F>(&mut self, column: Column<F>) -> &mut Self {
        self.order_by(column, true)
    }

    /// ORDER BY DESC（类型安全列引用）。
    pub fn order_by_desc<F>(&mut self, column: Column<F>) -> &mut Self {
        self.order_by(column, false)
    }

    /// HAVING 子句。
    ///
    /// 对应 Java `AbstractWrapper.having(...)`
    pub fn having(&mut self, sql: &str) -> &mut Self {
        self.func.having.push(sql.to_string());
        self
    }

    // ── Nested / OR / Join ────────────────────────────────────────────

    /// 下一个条件用 OR 连接。
    ///
    /// 对应 Java `AbstractWrapper.or()`
    pub fn or(&mut self) -> &mut Self {
        self.inner.segments.set_next_or(true);
        self
    }

    /// AND (sub-conditions)
    ///
    /// 对应 Java `AbstractWrapper.and(...)`
    pub fn and_group(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("({})", inner_sql));
        self
    }

    /// OR (sub-conditions)
    ///
    /// 对应 Java `AbstractWrapper.or(Consumer)` — 字符串版本
    pub fn or_group(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment_or(format!("({})", inner_sql));
        self
    }

    /// 嵌套条件 `AND (conditions)`
    ///
    /// 对应 Java `AbstractWrapper.nested(...)`
    pub fn nested(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("({})", inner_sql));
        self
    }

    /// NOT (sub-conditions)
    ///
    /// 对应 Java `AbstractWrapper.not(...)`
    pub fn not_group(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("NOT ({})", inner_sql));
        self
    }

    /// 自定义 SQL 片段拼接。
    ///
    /// 对应 Java `AbstractWrapper.apply(...)`
    pub fn apply(&mut self, sql: &str) -> &mut Self {
        self.inner.add_fragment(sql.to_string());
        self
    }

    /// EXISTS (subquery)
    ///
    /// 对应 Java `AbstractWrapper.exists(...)`
    pub fn exists(&mut self, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("EXISTS ({})", sql));
        self
    }

    /// NOT EXISTS (subquery)
    ///
    /// 对应 Java `AbstractWrapper.notExists(...)`
    pub fn not_exists(&mut self, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("NOT EXISTS ({})", sql));
        self
    }

    /// 追加 SQL 到末尾（如 `LIMIT 1`）。
    ///
    /// 对应 Java `AbstractWrapper.last(...)`
    pub fn last(&mut self, sql: &str) -> &mut Self {
        self.inner.sql_last = sql.to_string();
        self
    }

    /// SQL 注释（前置）。
    ///
    /// 对应 Java `AbstractWrapper.comment(...)`
    pub fn comment(&mut self, sql: &str) -> &mut Self {
        self.inner.sql_comment = sql.to_string();
        self
    }

    /// SQL first（最前置）。
    ///
    /// 对应 Java `AbstractWrapper.first(...)`
    pub fn first(&mut self, sql: &str) -> &mut Self {
        self.inner.sql_first = sql.to_string();
        self
    }

    // ── SQL 构建 ──────────────────────────────────────────────────────

    /// 构建完整 SELECT SQL。
    ///
    /// 对应 Java `QueryWrapper.getSqlSelect()` → `SqlUtils.SqlSegment`
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

    /// 构建 COUNT SQL（用于分页）。
    pub fn build_count_sql(&self, table_name: &str) -> String {
        let where_clause = self.inner.build_where();
        let mut sql = format!("SELECT COUNT(*) AS total FROM {}{}", table_name, where_clause);
        if !self.func.build_group_by().is_empty() {
            sql.push_str(&self.func.build_group_by());
            sql.push_str(&self.func.build_having());
            sql = format!("SELECT COUNT(*) AS total FROM ({}) t", sql);
        }
        sql
    }

    /// 返回绑定参数。
    pub fn params(&self) -> &[Value] {
        self.inner.params()
    }

    /// 清空所有条件。
    pub fn clear(&mut self) {
        self.inner.clear();
        self.func = FuncSegments::default();
        self.select_columns.clear();
    }
}
