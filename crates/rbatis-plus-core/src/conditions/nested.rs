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

/// Join / raw-SQL condition methods — 已迁移至 `join.rs`。
///
/// 原始 Java `Join<Children>` trait 现在定义在 `conditions::join` 模块，
/// 与 `Nested` trait 分离，实现"一个文件一个 trait"的 Rust 惯例。

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

// Join trait 已迁移至 join.rs
