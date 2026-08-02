//! SQL 片段接口（用于 MergeSegments 存储条件片段）。
//!
//! 对应 Java：`com.baomidou.mybatisplus.core.conditions.ISqlSegment`
//! 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/conditions/ISqlSegment.java`
//!
//! Java 中 `ISqlSegment` 是 MergeSegments 的元素接口；Rust 端
//! MergeSegments 用 `Vec<(SqlType, SharedString)>` 替代，此文件保留
//! 以维持模块结构对齐，并提供 SQL 类型枚举。

/// SQL 片段类型（用于 MergeSegments 中的元素分类）。
///
/// 对应 Java：`ISqlSegment` 的隐式类型分类（由 ISqlSegment 接口方法推导）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlType {
    /// AND 连接片段
    And,
    /// OR 连接片段
    Or,
    /// 普通条件片段
    Normal,
}

/// SQL 片段接口（Rust 端用 `Vec<(SqlType, SharedString)>` 替代此 trait）。
///
/// 对应 Java：`com.baomidou.mybatisplus.core.conditions.ISqlSegment`
pub trait ISqlSegment: std::fmt::Display {
    /// 获取 SQL 片段类型
    fn sql_type(&self) -> SqlType;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_type_values() {
        assert_ne!(SqlType::And, SqlType::Or);
        assert_ne!(SqlType::Or, SqlType::Normal);
    }
}
