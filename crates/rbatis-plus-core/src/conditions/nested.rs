// Source: mybatis-plus-core/.../conditions/interfaces/Nested.java
// Source: mybatis-plus-core/.../conditions/interfaces/Join.java

use super::abstract_wrapper::AbstractWrapper;

/// Nested condition methods: AND, OR, nested groups.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.conditions.interfaces.Nested<Children>`
/// and `Join<Children>`.
pub trait Nested {
    /// Bare `OR` connector — changes the next condition's connector to OR.
    ///
    /// 主动调用 OR 紧接着下一个条件用 OR 连接
    fn or(&mut self) -> &mut Self;

    /// `AND (...)` — adds a grouped AND block.
    ///
    /// AND 嵌套
    fn and_group(&mut self, inner_sql: &str) -> &mut Self;

    /// `OR (...)` — adds a grouped OR block.
    ///
    /// OR 嵌套
    fn or_group(&mut self, inner_sql: &str) -> &mut Self;

    /// `(inner_sql)` — nested without explicit AND/OR (uses current connector).
    ///
    /// 嵌套 (内层 SQL)
    fn nested(&mut self, inner_sql: &str) -> &mut Self;

    /// `NOT (...)` — negation group.
    ///
    /// NOT 嵌套
    fn not_group(&mut self, inner_sql: &str) -> &mut Self;
}

/// Join / raw-SQL condition methods.
///
/// Mirrors Java `com.baomidou.mybatisplus.core.conditions.interfaces.Join<Children>`.
pub trait Join {
    /// Append a raw SQL fragment (no parameter binding).
    ///
    /// 拼接 SQL (无参数绑定)
    fn apply(&mut self, sql: &str) -> &mut Self;

    /// `EXISTS (subquery)`
    ///
    /// EXISTS (子查询)
    fn exists(&mut self, sql: &str) -> &mut Self;

    /// `NOT EXISTS (subquery)`
    ///
    /// NOT EXISTS (子查询)
    fn not_exists(&mut self, sql: &str) -> &mut Self;

    /// Append raw SQL to the very end (e.g. `LIMIT 1`).
    ///
    /// 在 SQL 末尾追加 (如 LIMIT 1)
    fn last(&mut self, sql: &str) -> &mut Self;

    /// Prepend a SQL comment.
    ///
    /// 添加 SQL 注释
    fn comment(&mut self, sql: &str) -> &mut Self;

    /// Prepend raw SQL before the main query.
    ///
    /// 在 SQL 开头追加
    fn first(&mut self, sql: &str) -> &mut Self;
}

// ── Blanket impl for AbstractWrapper ───────────────────────────────────

impl Nested for AbstractWrapper {
    fn or(&mut self) -> &mut Self {
        // The next fragment will be connected with OR.
        // We set a flag via a zero-width marker; the MergeSegments
        // picks up the connector from the last `add_*` call.
        // For simplicity, the next `add_fragment_or` call will handle it.
        // Here we just record that the connector should be OR.
        // Since MergeSegments tracks last_connector, we flip it.
        self.segments.set_next_or(true);
        self
    }

    fn and_group(&mut self, inner_sql: &str) -> &mut Self {
        self.add_fragment(format!("({})", inner_sql));
        self
    }

    fn or_group(&mut self, inner_sql: &str) -> &mut Self {
        self.add_fragment_or(format!("({})", inner_sql));
        self
    }

    fn nested(&mut self, inner_sql: &str) -> &mut Self {
        self.add_fragment(format!("({})", inner_sql));
        self
    }

    fn not_group(&mut self, inner_sql: &str) -> &mut Self {
        self.add_fragment(format!("NOT ({})", inner_sql));
        self
    }
}

impl Join for AbstractWrapper {
    fn apply(&mut self, sql: &str) -> &mut Self {
        self.add_fragment(sql.to_string());
        self
    }

    fn exists(&mut self, sql: &str) -> &mut Self {
        self.add_fragment(format!("EXISTS ({})", sql));
        self
    }

    fn not_exists(&mut self, sql: &str) -> &mut Self {
        self.add_fragment(format!("NOT EXISTS ({})", sql));
        self
    }

    fn last(&mut self, sql: &str) -> &mut Self {
        self.sql_last = sql.to_string();
        self
    }

    fn comment(&mut self, sql: &str) -> &mut Self {
        self.sql_comment = sql.to_string();
        self
    }

    fn first(&mut self, sql: &str) -> &mut Self {
        self.sql_first = sql.to_string();
        self
    }
}
