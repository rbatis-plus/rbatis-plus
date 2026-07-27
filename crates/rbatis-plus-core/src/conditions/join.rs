//! Join trait — 子查询/连接操作。
//!
//! 对应 Java：`com.baomidou.mybatisplus.core.conditions.interfaces.Join<Children>`
//! 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/conditions/interfaces/Join.java`
//!
//! 提供 `apply`、`exists`、`not_exists`、`last`、`comment`、`first` 等连接/后缀操作。
//!
//! # Join 方法对照
//!
//! | Java Join 方法 | Rust 方法 | 说明 |
//! |---|---|---|
//! | `apply(sql, params...)` | `apply(sql)` | 追加原生 SQL 片段（防注入需调用方自行保证） |
//! | `exists(sql)` | `exists(sql)` | EXISTS 子查询 |
//! | `notExists(sql)` | `not_exists(sql)` | NOT EXISTS 子查询 |
//! | `last(sql)` | `last(sql)` | SQL 末尾追加（如 `LIMIT 1`） |
//! | `comment(sql)` | `comment(sql)` | SQL 注释（`/* ... */`） |
//! | `first(sql)` | `first(sql)` | SQL 首部插入（如 hint） |

use super::abstract_wrapper::AbstractWrapper;

/// Join / 连接操作 trait。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.conditions.interfaces.Join<Children>`
/// 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/conditions/interfaces/Join.java`
pub trait Join {
    /// 追加原生 SQL WHERE 片段（防注入需调用方自行保证）。
    ///
    /// 对应 Java：`Join.apply(boolean condition, String sql, Object... values)`
    fn apply(&mut self, sql: &str) -> &mut Self;

    /// EXISTS 子查询。
    ///
    /// 对应 Java：`Join.exists(boolean condition, String sql, Object... values)`
    fn exists(&mut self, sql: &str) -> &mut Self;

    /// NOT EXISTS 子查询。
    ///
    /// 对应 Java：`Join.notExists(boolean condition, String sql, Object... values)`
    fn not_exists(&mut self, sql: &str) -> &mut Self;

    /// SQL 末尾追加片段（如 `LIMIT 1`、`FOR UPDATE` 等）。
    ///
    /// 对应 Java：`Join.last(boolean condition, String sql)`
    fn last(&mut self, sql: &str) -> &mut Self;

    /// SQL 首部插入片段（如 hint）。
    ///
    /// 对应 Java：`Join.first(boolean condition, String sql)`
    fn first(&mut self, sql: &str) -> &mut Self;

    /// 添加 SQL 注释（`/* ... */`）。
    ///
    /// 对应 Java：`Join.comment(boolean condition, String sql)`
    fn comment(&mut self, sql: &str) -> &mut Self;
}

// ── Blanket impl for AbstractWrapper ───────────────────────────────────

impl Join for AbstractWrapper {
    fn apply(&mut self, sql: &str) -> &mut Self {
        self.add_fragment(sql);
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

    fn first(&mut self, sql: &str) -> &mut Self {
        self.sql_first = sql.to_string();
        self
    }

    fn comment(&mut self, sql: &str) -> &mut Self {
        self.sql_comment = format!("/* {} */", sql);
        self
    }
}
