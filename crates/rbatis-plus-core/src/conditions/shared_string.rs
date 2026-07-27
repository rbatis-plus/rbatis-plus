//! 共享字符串包装器（用于 MergeSegments 中共享字符串引用）。
//!
//! 对应 Java：`com.baomidou.mybatisplus.core.conditions.segments.SharedString`
//! 文件来源参考：`mybatis-plus-core/src/main/java/com/baomidou/mybatisplus/core/conditions/segments/SharedString.java`
//!
//! 目的是在 SQL 片段存储中共享字符串引用，避免多次 clone。
//! Rust 端直接用 `String`，此模块保留以维持模块结构对齐。

/// SQL 共享字符串（Java SharedString 的 Rust 对位）。
///
/// Java 版本用 `AtomicReference<String>` 做共享引用；Rust 直接用 `String`（编译器
/// 借用检查已保证引用安全，不需要额外的原子包装）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedString(String);

impl SharedString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<&str> for SharedString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for SharedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_string_creation() {
        let s = SharedString::new("hello");
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.to_string(), "hello");
    }

    #[test]
    fn shared_string_from_str() {
        let s: SharedString = "test".into();
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    fn shared_string_clone() {
        let s1 = SharedString::new("abc");
        let s2 = s1.clone();
        assert_eq!(s1, s2);
    }
}
