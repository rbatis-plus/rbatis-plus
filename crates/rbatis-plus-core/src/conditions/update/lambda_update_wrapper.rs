// Source: mybatis-plus-core/.../conditions/update/LambdaUpdateWrapper.java

use super::super::abstract_wrapper::AbstractWrapper;
use super::super::func::FuncSegments;
use super::super::query::column::Column;
use rbs::Value;
use std::marker::PhantomData;

/// 基于类型安全列引用的更新构建器（对标 Java `LambdaUpdateWrapper<T>`）。
///
/// 使用 `Column<F>` 替代字符串列名，编译期保证列名正确性。
/// 同时支持 WHERE 条件和 SET 子句，可构建 UPDATE 和 DELETE 语句。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.core.conditions.update.LambdaUpdateWrapper<T>`
/// - `com.baomidou.mybatisplus.core.conditions.update.Update<T>`
///
/// # Example
///
/// ```ignore
/// use rbatis_plus::core::conditions::update::LambdaUpdateWrapper;
///
/// let mut w = LambdaUpdateWrapper::<User>::new();
/// w.set(User::column_name(), "Bob")
///  .set_incr_by(User::column_age(), 1)
///  .eq(User::column_id(), 42i64);
///
/// let sql = w.build_update_sql("sys_user");
/// // UPDATE sys_user SET name = 'Bob', age = age + 1 WHERE id = 42
/// ```
#[derive(Debug, Clone)]
pub struct LambdaUpdateWrapper<T> {
    /// 内部条件构建状态（WHERE 子句）。
    pub inner: AbstractWrapper,
    /// GROUP BY / ORDER BY / HAVING 状态。
    pub func: FuncSegments,
    /// SET 子句片段列表。
    set_fragments: Vec<String>,
    /// 实体类型幽灵标记。
    _phantom: PhantomData<T>,
}

impl<T> Default for LambdaUpdateWrapper<T> {
    fn default() -> Self {
        Self {
            inner: AbstractWrapper::default(),
            func: FuncSegments::default(),
            set_fragments: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T> LambdaUpdateWrapper<T> {
    /// 创建空的 Lambda 更新构建器。
    pub fn new() -> Self {
        Self::default()
    }

    // ── SET 子句（类型安全列引用）───────────────────────────────────

    /// `SET column = value`（类型安全列引用）
    ///
    /// 对应 Java `Update.set(boolean condition, R column, Object val)`
    pub fn set<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        let literal = AbstractWrapper::format_eq(column.name(), &v);
        self.set_fragments.push(literal);
        self
    }

    /// 条件 SET — 仅当 `condition` 为 true 时生效。
    pub fn if_set<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.set(column, value); }
        self
    }

    /// `SET column = column + delta`（类型安全列引用，自增）
    ///
    /// 对应 Java `Update.setIncrBy(...)`
    pub fn set_incr_by<F>(&mut self, column: Column<F>, delta: i64) -> &mut Self {
        let name = column.name();
        self.set_fragments.push(format!("{} = {} + {}", name, name, delta));
        self
    }

    /// `SET column = column - delta`（类型安全列引用，自减）
    ///
    /// 对应 Java `Update.setDecrBy(...)`
    pub fn set_decr_by<F>(&mut self, column: Column<F>, delta: i64) -> &mut Self {
        let name = column.name();
        self.set_fragments.push(format!("{} = {} - {}", name, name, delta));
        self
    }

    /// 追加原始 SET 片段（如 `"age = age + 1"`、`"update_time = now()"`）。
    ///
    /// 对应 Java `Update.setSql(...)`
    pub fn set_sql(&mut self, sql: &str) -> &mut Self {
        self.set_fragments.push(sql.to_string());
        self
    }

    // ── Compare（类型安全列引用 WHERE 条件）─────────────────────────

    /// `WHERE column = value`
    ///
    /// 对应 Java `AbstractWrapper.eq(...)`
    pub fn eq<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_eq(column.name(), &v));
        self
    }

    /// 条件 eq。
    pub fn if_eq<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.eq(column, value); }
        self
    }

    /// `WHERE column <> value`
    ///
    /// 对应 Java `AbstractWrapper.ne(...)`
    pub fn ne<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_ne(column.name(), &v));
        self
    }

    /// 条件 ne。
    pub fn if_ne<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.ne(column, value); }
        self
    }

    /// `WHERE column > value`
    ///
    /// 对应 Java `AbstractWrapper.gt(...)`
    pub fn gt<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_gt(column.name(), &v));
        self
    }

    /// 条件 gt。
    pub fn if_gt<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.gt(column, value); }
        self
    }

    /// `WHERE column >= value`
    ///
    /// 对应 Java `AbstractWrapper.ge(...)`
    pub fn ge<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_ge(column.name(), &v));
        self
    }

    /// 条件 ge。
    pub fn if_ge<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.ge(column, value); }
        self
    }

    /// `WHERE column < value`
    ///
    /// 对应 Java `AbstractWrapper.lt(...)`
    pub fn lt<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_lt(column.name(), &v));
        self
    }

    /// 条件 lt。
    pub fn if_lt<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.lt(column, value); }
        self
    }

    /// `WHERE column <= value`
    ///
    /// 对应 Java `AbstractWrapper.le(...)`
    pub fn le<F>(&mut self, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        let v = value.into();
        self.inner.add_fragment(AbstractWrapper::format_le(column.name(), &v));
        self
    }

    /// 条件 le。
    pub fn if_le<F>(&mut self, condition: bool, column: Column<F>, value: impl Into<Value>) -> &mut Self {
        if condition { self.le(column, value); }
        self
    }

    /// `WHERE column BETWEEN val1 AND val2`
    ///
    /// 对应 Java `AbstractWrapper.between(...)`
    pub fn between<F>(&mut self, column: Column<F>, val1: impl Into<Value>, val2: impl Into<Value>) -> &mut Self {
        let v1 = val1.into();
        let v2 = val2.into();
        self.inner.add_fragment(AbstractWrapper::format_between(column.name(), &v1, &v2));
        self
    }

    /// 条件 between。
    pub fn if_between<F>(
        &mut self, condition: bool, column: Column<F>,
        val1: impl Into<Value>, val2: impl Into<Value>,
    ) -> &mut Self {
        if condition { self.between(column, val1, val2); }
        self
    }

    /// `WHERE column NOT BETWEEN val1 AND val2`
    ///
    /// 对应 Java `AbstractWrapper.notBetween(...)`
    pub fn not_between<F>(&mut self, column: Column<F>, val1: impl Into<Value>, val2: impl Into<Value>) -> &mut Self {
        let v1 = val1.into();
        let v2 = val2.into();
        self.inner.add_fragment(AbstractWrapper::format_not_between(column.name(), &v1, &v2));
        self
    }

    /// 条件 not_between。
    pub fn if_not_between<F>(
        &mut self, condition: bool, column: Column<F>,
        val1: impl Into<Value>, val2: impl Into<Value>,
    ) -> &mut Self {
        if condition { self.not_between(column, val1, val2); }
        self
    }

    /// `WHERE column = value OR column IS NULL`
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

    // ── Func（类型安全列引用 WHERE 条件）────────────────────────────

    /// `WHERE column LIKE '%value%'`
    ///
    /// 对应 Java `AbstractWrapper.like(...)`
    pub fn like<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like(column.name(), value));
        self
    }

    /// 条件 like。
    pub fn if_like<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.like(column, value); }
        self
    }

    /// `WHERE column NOT LIKE '%value%'`
    ///
    /// 对应 Java `AbstractWrapper.notLike(...)`
    pub fn not_like<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_not_like(column.name(), value));
        self
    }

    /// 条件 not_like。
    pub fn if_not_like<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.not_like(column, value); }
        self
    }

    /// `WHERE column LIKE 'value%'`（右匹配）
    ///
    /// 对应 Java `AbstractWrapper.likeRight(...)`
    pub fn like_right<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like_right(column.name(), value));
        self
    }

    /// 条件 like_right。
    pub fn if_like_right<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.like_right(column, value); }
        self
    }

    /// `WHERE column LIKE '%value'`（左匹配）
    ///
    /// 对应 Java `AbstractWrapper.likeLeft(...)`
    pub fn like_left<F>(&mut self, column: Column<F>, value: &str) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_like_left(column.name(), value));
        self
    }

    /// 条件 like_left。
    pub fn if_like_left<F>(&mut self, condition: bool, column: Column<F>, value: &str) -> &mut Self {
        if condition { self.like_left(column, value); }
        self
    }

    /// `WHERE column IS NULL`
    ///
    /// 对应 Java `AbstractWrapper.isNull(...)`
    pub fn is_null<F>(&mut self, column: Column<F>) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_is_null(column.name()));
        self
    }

    /// 条件 is_null。
    pub fn if_is_null<F>(&mut self, condition: bool, column: Column<F>) -> &mut Self {
        if condition { self.is_null(column); }
        self
    }

    /// `WHERE column IS NOT NULL`
    ///
    /// 对应 Java `AbstractWrapper.isNotNull(...)`
    pub fn is_not_null<F>(&mut self, column: Column<F>) -> &mut Self {
        self.inner.add_fragment(AbstractWrapper::format_is_not_null(column.name()));
        self
    }

    /// 条件 is_not_null。
    pub fn if_is_not_null<F>(&mut self, condition: bool, column: Column<F>) -> &mut Self {
        if condition { self.is_not_null(column); }
        self
    }

    /// `WHERE column IN (v1, v2, ...)`
    ///
    /// 对应 Java `AbstractWrapper.in(...)`
    pub fn in_values<F>(&mut self, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if !values.is_empty() {
            self.inner.add_fragment(AbstractWrapper::format_in(column.name(), &values));
        }
        self
    }

    /// 条件 in_values。
    pub fn if_in_values<F>(&mut self, condition: bool, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if condition { self.in_values(column, values); }
        self
    }

    /// `WHERE column NOT IN (v1, v2, ...)`
    ///
    /// 对应 Java `AbstractWrapper.notIn(...)`
    pub fn not_in<F>(&mut self, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if !values.is_empty() {
            self.inner.add_fragment(AbstractWrapper::format_not_in(column.name(), &values));
        }
        self
    }

    /// 条件 not_in。
    pub fn if_not_in<F>(&mut self, condition: bool, column: Column<F>, values: Vec<Value>) -> &mut Self {
        if condition { self.not_in(column, values); }
        self
    }

    /// `WHERE column IN (subquery)`
    ///
    /// 对应 Java `AbstractWrapper.inSql(...)`
    pub fn in_sql<F>(&mut self, column: Column<F>, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("{} IN ({})", column.name(), sql));
        self
    }

    /// `WHERE column NOT IN (subquery)`
    ///
    /// 对应 Java `AbstractWrapper.notInSql(...)`
    pub fn not_in_sql<F>(&mut self, column: Column<F>, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("{} NOT IN ({})", column.name(), sql));
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
    pub fn and_group(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("({})", inner_sql));
        self
    }

    /// OR (sub-conditions)
    pub fn or_group(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment_or(format!("({})", inner_sql));
        self
    }

    /// 嵌套条件 `AND (conditions)`
    pub fn nested(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("({})", inner_sql));
        self
    }

    /// NOT (sub-conditions)
    pub fn not_group(&mut self, inner_sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("NOT ({})", inner_sql));
        self
    }

    /// 自定义 SQL 片段拼接。
    pub fn apply(&mut self, sql: &str) -> &mut Self {
        self.inner.add_fragment(sql.to_string());
        self
    }

    /// EXISTS (subquery)
    pub fn exists(&mut self, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("EXISTS ({})", sql));
        self
    }

    /// NOT EXISTS (subquery)
    pub fn not_exists(&mut self, sql: &str) -> &mut Self {
        self.inner.add_fragment(format!("NOT EXISTS ({})", sql));
        self
    }

    /// 追加 SQL 到末尾（如 `LIMIT 1`）。
    pub fn last(&mut self, sql: &str) -> &mut Self {
        self.inner.sql_last = sql.to_string();
        self
    }

    /// SQL 注释（前置）。
    pub fn comment(&mut self, sql: &str) -> &mut Self {
        self.inner.sql_comment = sql.to_string();
        self
    }

    /// SQL first（最前置）。
    pub fn first(&mut self, sql: &str) -> &mut Self {
        self.inner.sql_first = sql.to_string();
        self
    }

    // ── SQL 构建 ──────────────────────────────────────────────────────

    /// 构建 SET 子句字符串。
    pub fn build_set(&self) -> String {
        if self.set_fragments.is_empty() {
            String::new()
        } else {
            self.set_fragments.join(", ")
        }
    }

    /// 构建完整 UPDATE SQL。
    ///
    /// 对应 Java `UpdateWrapper.getSqlSet()` + `AbstractWrapper.getCustomSqlSegment()`
    pub fn build_update_sql(&self, table_name: &str) -> String {
        let set_clause = self.build_set();
        let where_clause = self.inner.build_where();

        let mut sql = String::new();

        // sql_first（最前置）
        if !self.inner.sql_first.is_empty() {
            sql.push_str(&self.inner.sql_first);
            sql.push(' ');
        }

        // sql_comment（前置注释）
        if !self.inner.sql_comment.is_empty() {
            sql.push_str(&self.inner.sql_comment);
            sql.push(' ');
        }

        sql.push_str(&format!("UPDATE {} SET {}{}", table_name, set_clause, where_clause));

        // sql_last（末尾追加）
        if !self.inner.sql_last.is_empty() {
            sql.push(' ');
            sql.push_str(&self.inner.sql_last);
        }

        sql
    }

    /// 构建 DELETE SQL（使用 WHERE 条件）。
    ///
    /// 对应 Java `DeleteWrapper`（实际复用 UpdateWrapper 的 WHERE）
    pub fn build_delete_sql(&self, table_name: &str) -> String {
        let where_clause = self.inner.build_where();

        let mut sql = String::new();

        if !self.inner.sql_first.is_empty() {
            sql.push_str(&self.inner.sql_first);
            sql.push(' ');
        }

        if !self.inner.sql_comment.is_empty() {
            sql.push_str(&self.inner.sql_comment);
            sql.push(' ');
        }

        sql.push_str(&format!("DELETE FROM {}{}", table_name, where_clause));

        if !self.inner.sql_last.is_empty() {
            sql.push(' ');
            sql.push_str(&self.inner.sql_last);
        }

        sql
    }

    /// 返回绑定参数。
    pub fn params(&self) -> &[Value] {
        self.inner.params()
    }

    /// 清空所有条件和 SET 片段。
    pub fn clear(&mut self) {
        self.inner.clear();
        self.func = FuncSegments::default();
        self.set_fragments.clear();
    }
}
