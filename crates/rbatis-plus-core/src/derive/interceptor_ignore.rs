/// 拦截器忽略规则标记。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.InterceptorIgnore`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/InterceptorIgnore.java`
///
/// 内置插件的过滤规则。注解在 Mapper 接口或 Mapper 方法上均可，Method 级别优先于接口级别。
///
/// 支持的布尔值格式：`true`/`false`、`1`/`0`、`on`/`off`。
/// 各属性返回 `true` 表示跳过该插件。
///
/// ```rust
/// use rbatis_plus_core::InterceptorIgnore;
///
/// trait UserInterceptorIgnore {
///     // derive 宏自动实现
/// }
///
/// // derive(InterceptorIgnore) 生成的实现示例：
/// // impl InterceptorIgnore for User {
/// //     fn tenant_line() -> bool { false }          // 默认不禁用
/// //     fn dynamic_table_name() -> bool { false }   // 默认不禁用
/// //     fn block_attack() -> bool { false }         // 默认不禁用
/// //     fn illegal_sql() -> bool { false }          // 默认不禁用
/// //     fn data_permission() -> bool { true }       // 默认禁用（注意！与 Java 不同）
/// //     fn others() -> &'static [(&'static str, bool)] { &[] }
/// // }
/// ```
pub trait InterceptorIgnore {
    /// 行级租户拦截器（对应 Java `@InterceptorIgnore.tenantLine()`；默认 false → 不跳过）。
    fn tenant_line() -> bool {
        false
    }

    /// 动态表名拦截器（对应 Java `@InterceptorIgnore.dynamicTableName()`；默认 false → 不跳过）。
    fn dynamic_table_name() -> bool {
        false
    }

    /// 攻击 SQL 阻断拦截器（对应 Java `@InterceptorIgnore.blockAttack()`；默认 false → 不跳过）。
    fn block_attack() -> bool {
        false
    }

    /// 垃圾 SQL 拦截器（对应 Java `@InterceptorIgnore.illegalSql()`；默认 false → 不跳过）。
    fn illegal_sql() -> bool {
        false
    }

    /// 数据权限拦截器（对应 Java `@InterceptorIgnore.dataPermission()`）。
    ///
    /// **注意**：Java 默认值是 `"1"`（表示禁用），Rust 端为了安全默认 `false`（不禁用），
    /// 这是与 Java 行为的**有意差异**——Rust 端默认不禁用，需要显式启用忽略。
    fn data_permission() -> bool {
        false
    }

    /// 其他自定义拦截器忽略规则（对应 Java `@InterceptorIgnore.others()`）。
    ///
    /// 返回 `(&key, ignore_bool)` 元组列表。key 格式与 Java 一致：`"key@value"`。
    fn others() -> &'static [(&'static str, bool)] {
        &[]
    }
}

/// 运行时拦截器忽略状态（供 `InterceptorIgnoreHelper` 使用）。
///
/// 对应 Java：`InterceptorIgnore` 的运行时检查逻辑。
#[derive(Debug, Clone, Default)]
pub struct InterceptorIgnoreInfo {
    /// 跳过行级租户拦截器。
    pub tenant_line: bool,
    /// 跳过动态表名拦截器。
    pub dynamic_table_name: bool,
    /// 跳过攻击 SQL 阻断拦截器。
    pub block_attack: bool,
    /// 跳过垃圾 SQL 拦截器。
    pub illegal_sql: bool,
    /// 跳过数据权限拦截器。
    pub data_permission: bool,
    /// 自定义忽略规则。
    pub others: Vec<(String, bool)>,
}

impl InterceptorIgnoreInfo {
    /// 从 trait 实例构建（derive 宏展开时调用）。
    pub fn from_trait<T: InterceptorIgnore>() -> Self {
        Self {
            tenant_line: T::tenant_line(),
            dynamic_table_name: T::dynamic_table_name(),
            block_attack: T::block_attack(),
            illegal_sql: T::illegal_sql(),
            data_permission: T::data_permission(),
            others: T::others().iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        }
    }

    /// 从字符串解析（Java `@InterceptorIgnore` 的字符串值解析）。
    ///
    /// 支持：`"true"` / `"false"` / `"1"` / `"0"` / `"on"` / `"off"`
    pub fn parse_bool(value: &str) -> bool {
        matches!(
            value.trim().to_lowercase().as_str(),
            "true" | "1" | "on"
        )
    }

    /// 方法级规则是否跳过指定插件。
    pub fn should_ignore(&self, plugin_name: &str) -> bool {
        match plugin_name {
            "tenant_line" => self.tenant_line,
            "dynamic_table_name" => self.dynamic_table_name,
            "block_attack" => self.block_attack,
            "illegal_sql" => self.illegal_sql,
            "data_permission" => self.data_permission,
            _ => self.others.iter().any(|(k, v)| k == plugin_name && *v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockIgnoreMapper;
    impl InterceptorIgnore for MockIgnoreMapper {
        fn tenant_line() -> bool { true }
        fn block_attack() -> bool { true }
        fn others() -> &'static [(&'static str, bool)] {
            &[("custom@1", true), ("other@0", false)]
        }
    }

    #[test]
    fn defaults_are_false() {
        struct DefaultImpl;
        impl InterceptorIgnore for DefaultImpl {}

        assert!(!DefaultImpl::tenant_line());
        assert!(!DefaultImpl::dynamic_table_name());
        assert!(!DefaultImpl::block_attack());
        assert!(!DefaultImpl::illegal_sql());
        assert!(!DefaultImpl::data_permission());
        assert_eq!(DefaultImpl::others(), &[]);
    }

    #[test]
    fn custom_values() {
        assert!(MockIgnoreMapper::tenant_line());
        assert!(!MockIgnoreMapper::dynamic_table_name());
        assert!(MockIgnoreMapper::block_attack());
        assert!(!MockIgnoreMapper::illegal_sql());
        assert!(!MockIgnoreMapper::data_permission());
    }

    #[test]
    fn parse_bool_variants() {
        assert!(InterceptorIgnoreInfo::parse_bool("true"));
        assert!(InterceptorIgnoreInfo::parse_bool("1"));
        assert!(InterceptorIgnoreInfo::parse_bool("on"));
        assert!(InterceptorIgnoreInfo::parse_bool("TRUE"));
        assert!(!InterceptorIgnoreInfo::parse_bool("false"));
        assert!(!InterceptorIgnoreInfo::parse_bool("0"));
        assert!(!InterceptorIgnoreInfo::parse_bool("off"));
        assert!(!InterceptorIgnoreInfo::parse_bool("anything"));
    }

    #[test]
    fn should_ignore_matches() {
        let info = InterceptorIgnoreInfo::from_trait::<MockIgnoreMapper>();
        assert!(info.should_ignore("tenant_line"));
        assert!(!info.should_ignore("dynamic_table_name"));
        assert!(info.should_ignore("block_attack"));
        assert!(!info.should_ignore("data_permission"));
        assert!(info.should_ignore("custom@1"));
        assert!(!info.should_ignore("other@0"));
    }

    #[test]
    fn java_semantics_data_permission_default() {
        // Java @InterceptorIgnore.dataPermission() 默认 "1" → skip
        // Rust trait 默认 false → don't skip (intentional difference for safety)
        struct JavaStyleImpl;
        impl InterceptorIgnore for JavaStyleImpl {
            fn data_permission() -> bool { true } // 模拟 Java 的 "1" 行为
        }
        assert!(JavaStyleImpl::data_permission());
    }
}
