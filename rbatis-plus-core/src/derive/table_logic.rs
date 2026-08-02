/// 逻辑删除字段标记。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.TableLogic`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/TableLogic.java`
///
/// ```rust
/// use rbatis_plus_core::TableLogic;
///
/// trait UserTableLogic {
///     // derive 宏自动实现
/// }
///
/// // derive(TableLogic) 生成的实现示例：
/// // impl TableLogic for User {
/// //     fn logic_column() -> &'static str { "deleted" }
/// //     fn logic_not_delete_value() -> &'static str { "0" }  // 默认值，可被全局配置覆盖
/// //     fn logic_delete_value() -> &'static str { "1" }
/// // }
/// ```
pub trait TableLogic {
    /// 逻辑删除字段名。
    ///
    /// 对应 Java：`TableLogic.value()`（Lombok `@Getter`）。
    fn logic_column() -> &'static str;

    /// 逻辑未删除值（默认值为 None，运行时从全局配置读取）。
    ///
    /// 对应 Java：`TableLogic.value()`（默认空串 → "用全局配置"）。
    /// Rust 端默认返回 `None`，由 `MetaObjectHandler` 或全局配置填充。
    ///
    /// # 注意
    /// 如果 derive 宏未指定 value，返回 `None`，表示使用 `GlobalConfig` 中的逻辑删除配置。
    fn logic_not_delete_value() -> Option<&'static str> { None }

    /// 逻辑删除值（默认值为 None，运行时从全局配置读取）。
    ///
    /// 对应 Java：`TableLogic.delval()`（默认空串 → "用全局配置"）。
    fn logic_delete_value() -> Option<&'static str> { None }
}

/// 逻辑删除全局配置（供 `MetaObjectHandler` / `TableFieldHelper` 使用）。
///
/// 当 `TableLogic::logic_not_delete_value()` 或 `logic_delete_value()` 返回
/// `None` 时，使用 `GlobalLogicDeleteConfig` 中的值。
///
/// 对应 Java：`MybatisGlobalConfiguration.getGlobalConfig().getDbConfig().getLogicDeleteValue()`
#[derive(Debug, Clone)]
pub struct GlobalLogicDeleteConfig {
    /// 逻辑未删除值（默认 "0"）。
    pub not_delete_value: String,
    /// 逻辑删除值（默认 "1"）。
    pub delete_value: String,
}

impl Default for GlobalLogicDeleteConfig {
    fn default() -> Self {
        Self {
            not_delete_value: "0".to_string(),
            delete_value: "1".to_string(),
        }
    }
}

impl GlobalLogicDeleteConfig {
    /// 解析某个 `TableLogic` 实现的实际值：优先用字段值，None 时回退全局配置。
    ///
    /// 对应 Java 的 MybatisGlobalConfiguration 合并逻辑。
    pub fn resolve_not_delete<'a>(field_value: Option<&'a str>, global: &'a Self) -> &'a str {
        match field_value {
            Some(v) if !v.is_empty() => v,
            _ => &global.not_delete_value,
        }
    }

    /// 解析逻辑删除值：优先用字段值，None 时回退全局配置。
    pub fn resolve_delete<'a>(field_value: Option<&'a str>, global: &'a Self) -> &'a str {
        match field_value {
            Some(v) if !v.is_empty() => v,
            _ => &global.delete_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockUserLogic;
    impl TableLogic for MockUserLogic {
        fn logic_column() -> &'static str { "deleted" }
        // 未指定 → 使用全局配置
    }

    #[derive(Debug)]
    struct MockCustomLogic;
    impl TableLogic for MockCustomLogic {
        fn logic_column() -> &'static str { "is_deleted" }
        fn logic_not_delete_value() -> Option<&'static str> { Some("false") }
        fn logic_delete_value() -> Option<&'static str> { Some("true") }
    }

    #[test]
    fn default_returns_none_for_values() {
        assert_eq!(MockUserLogic::logic_not_delete_value(), None);
        assert_eq!(MockUserLogic::logic_delete_value(), None);
    }

    #[test]
    fn custom_logic_returns_values() {
        assert_eq!(MockCustomLogic::logic_not_delete_value(), Some("false"));
        assert_eq!(MockCustomLogic::logic_delete_value(), Some("true"));
    }

    #[test]
    fn resolve_fallback_to_global() {
        let global = GlobalLogicDeleteConfig::default();
        assert_eq!(
            GlobalLogicDeleteConfig::resolve_not_delete(None, &global),
            "0"
        );
        assert_eq!(
            GlobalLogicDeleteConfig::resolve_delete(Some("x"), &global),
            "x"
        );
    }

    #[test]
    fn resolve_ignores_empty_string() {
        let global = GlobalLogicDeleteConfig {
            not_delete_value: "active".into(),
            delete_value: "removed".into(),
        };
        assert_eq!(
            GlobalLogicDeleteConfig::resolve_not_delete(Some(""), &global),
            "active"
        );
    }

    #[test]
    fn rust_trait_matches_java_semantics() {
        // Java TableLogic.value() 默认 "" → 用全局配置
        // Rust Option<&str> 默认 None → 用全局配置
        assert_eq!(MockUserLogic::logic_not_delete_value(), None);
        assert_eq!(MockUserLogic::logic_delete_value(), None);
    }
}
