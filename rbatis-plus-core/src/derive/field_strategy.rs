/// 字段策略枚举——决定字段何时被加入生成的 SQL。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.FieldStrategy`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/FieldStrategy.java`
///
/// 如果字段是基本数据类型则最终效果等同于 `Always`。
///
/// ```rust
/// use rbatis_plus_core::FieldStrategy;
///
/// let strategy = FieldStrategy::NotNull;
/// assert!(!matches!(strategy, FieldStrategy::Default));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum FieldStrategy {
    /// 任何时候都加入 SQL。
    ///
    /// 对应 Java：`FieldStrategy.ALWAYS`
    Always,
    /// 非 NULL 判断。
    ///
    /// 对应 Java：`FieldStrategy.NOT_NULL`
    NotNull,
    /// 非空判断（只对字符串类型字段有效，其他类型字段依然为非 NULL 判断）。
    ///
    /// 对应 Java：`FieldStrategy.NOT_EMPTY`
    NotEmpty,
    /// 默认的，一般只用于注解里：
    /// - 在全局配置里代表 `NOT_NULL`
    /// - 在注解里代表"跟随全局"
    ///
    /// 对应 Java：`FieldStrategy.DEFAULT`
    Default,
    /// 不加入 SQL。
    ///
    /// 对应 Java：`FieldStrategy.NEVER`
    Never,
}

impl Default for FieldStrategy {
    fn default() -> Self { Self::NotNull }
}

impl std::fmt::Display for FieldStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always  => write!(f, "ALWAYS"),
            Self::NotNull => write!(f, "NOT_NULL"),
            Self::NotEmpty => write!(f, "NOT_EMPTY"),
            Self::Default => write!(f, "DEFAULT"),
            Self::Never   => write!(f, "NEVER"),
        }
    }
}

impl serde::Serialize for FieldStrategy {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Always  => serializer.serialize_str("always"),
            Self::NotNull => serializer.serialize_str("not_null"),
            Self::NotEmpty => serializer.serialize_str("not_empty"),
            Self::Default => serializer.serialize_str("default"),
            Self::Never   => serializer.serialize_str("never"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FieldStrategy {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "always"   => Ok(Self::Always),
            "not_null" => Ok(Self::NotNull),
            "not_empty" => Ok(Self::NotEmpty),
            "default"  => Ok(Self::Default),
            "never"    => Ok(Self::Never),
            _ => Err(serde::de::Error::custom(format!("unknown FieldStrategy: {}", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_null() {
        assert_eq!(FieldStrategy::default(), FieldStrategy::NotNull);
    }

    #[test]
    fn display_matches_java_string() {
        assert_eq!(FieldStrategy::Always.to_string(), "ALWAYS");
        assert_eq!(FieldStrategy::Default.to_string(), "DEFAULT");
        assert_eq!(FieldStrategy::Never.to_string(), "NEVER");
    }

    #[test]
    fn serde_roundtrip() {
        let strategies = vec![
            FieldStrategy::Always,
            FieldStrategy::NotNull,
            FieldStrategy::NotEmpty,
            FieldStrategy::Default,
            FieldStrategy::Never,
        ];
        for s in strategies {
            let json = serde_json::to_string(&s).unwrap();
            let back: FieldStrategy = serde_json::from_str(&json).unwrap();
            assert_eq!(back, s);
        }
    }

    #[test]
    fn java_field_strategy_order_matches() {
        // Java FieldStrategy 枚举顺序：ALWAYS(0), NOT_NULL(1), NOT_EMPTY(2), DEFAULT(3), NEVER(4)
        assert!(FieldStrategy::Always < FieldStrategy::NotNull);
        assert!(FieldStrategy::NotNull < FieldStrategy::NotEmpty);
        assert!(FieldStrategy::NotEmpty < FieldStrategy::Default);
        assert!(FieldStrategy::Default < FieldStrategy::Never);
    }
}
