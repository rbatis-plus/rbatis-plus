//! 键值对结构（对标 Java `KeyValuePair`）。
//!
//! 用于 `I18nListResourceBundle` 构建时传递键值对数据。

/// 不可变键值对，持有单个国际化条目的 key 和 value。
///
/// 对应 Java：`java.util.spi.ResourceBundleEnumeration` 内部使用的 `KeyValuePair`，
/// 以及 MyBatis-Plus-Enhance 自定义的 `KeyValuePair`。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValuePair {
    /// 国际化键（如 "greeting"、"button.submit"）。
    pub key: String,
    /// 对应语言环境的翻译值（如 "你好"、"提交"）。
    pub value: String,
}

impl KeyValuePair {
    /// 创建新的键值对。
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl From<(&str, &str)> for KeyValuePair {
    fn from(tuple: (&str, &str)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl From<(String, String)> for KeyValuePair {
    fn from(tuple: (String, String)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_value_pair_creation() {
        let kvp = KeyValuePair::new("greeting", "你好");
        assert_eq!(kvp.key, "greeting");
        assert_eq!(kvp.value, "你好");
    }

    #[test]
    fn test_key_value_pair_from_str_tuple() {
        let kvp = KeyValuePair::from(("name", "名称"));
        assert_eq!(kvp.key, "name");
        assert_eq!(kvp.value, "名称");
    }

    #[test]
    fn test_key_value_pair_equality() {
        let a = KeyValuePair::new("k", "v");
        let b = KeyValuePair::new("k", "v");
        assert_eq!(a, b);
    }
}
