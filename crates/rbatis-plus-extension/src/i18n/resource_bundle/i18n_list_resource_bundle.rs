//! 国际化列表资源包（对标 Java `I18nListResourceBundle`）。
//!
//! 基于键值对列表构建的资源包实现，支持从外部数据源
//! （如数据库表、配置文件）加载翻译数据。

use std::collections::HashMap;

use super::key_value_pair::KeyValuePair;
use super::multiple_resource_bundle::ResourceBundle;
use super::resource_bundle_enumeration::ResourceBundleEnumeration;

/// 基于列表的资源包实现。
///
/// 对应 Java：MyBatis-Plus-Enhance 的 `I18nListResourceBundle`。
/// 内部使用 `HashMap` 存储键值对，O(1) 查找。
///
/// # 使用示例
///
/// ```ignore
/// let bundle = I18nListResourceBundle::from_pairs(vec![
///     KeyValuePair::new("greeting", "你好"),
///     KeyValuePair::new("farewell", "再见"),
/// ]);
/// assert_eq!(bundle.get_string("greeting"), Some("你好"));
/// ```
#[derive(Debug, Clone)]
pub struct I18nListResourceBundle {
    /// 语言环境标识（如 "zh_CN"、"en_US"）。
    locale: String,
    /// 键值对存储。
    entries: HashMap<String, String>,
}

impl I18nListResourceBundle {
    /// 创建指定语言环境的空资源包。
    pub fn new(locale: impl Into<String>) -> Self {
        Self {
            locale: locale.into(),
            entries: HashMap::new(),
        }
    }

    /// 从键值对列表构建资源包。
    pub fn from_pairs(locale: impl Into<String>, pairs: Vec<KeyValuePair>) -> Self {
        let entries = pairs.into_iter().map(|kvp| (kvp.key, kvp.value)).collect();
        Self {
            locale: locale.into(),
            entries,
        }
    }

    /// 从迭代器构建资源包。
    pub fn from_iter(
        locale: impl Into<String>,
        iter: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        Self {
            locale: locale.into(),
            entries: iter.into_iter().collect(),
        }
    }

    /// 获取语言环境标识。
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// 获取指定键的值。
    pub fn get_object(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    /// 获取指定键的字符串值。
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.get_object(key)
    }

    /// 插入或更新一个键值对。
    pub fn put(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    /// 批量插入键值对。
    pub fn put_all(&mut self, pairs: Vec<KeyValuePair>) {
        for kvp in pairs {
            self.entries.insert(kvp.key, kvp.value);
        }
    }

    /// 获取键枚举器。
    pub fn keys(&self) -> ResourceBundleEnumeration {
        ResourceBundleEnumeration::new(self.entries.keys().cloned())
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 条目数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否包含指定键。
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }
}

impl From<I18nListResourceBundle> for HashMap<String, String> {
    fn from(bundle: I18nListResourceBundle) -> Self {
        bundle.entries
    }
}

impl ResourceBundle for I18nListResourceBundle {
    fn get_object(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    fn keys(&self) -> ResourceBundleEnumeration {
        ResourceBundleEnumeration::new(self.entries.keys().cloned())
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_resource_bundle_from_pairs() {
        let bundle = I18nListResourceBundle::from_pairs(
            "zh_CN",
            vec![
                KeyValuePair::new("greeting", "你好"),
                KeyValuePair::new("farewell", "再见"),
            ],
        );
        assert_eq!(bundle.locale(), "zh_CN");
        assert_eq!(bundle.len(), 2);
        assert_eq!(bundle.get_string("greeting"), Some("你好"));
        assert_eq!(bundle.get_string("farewell"), Some("再见"));
        assert!(bundle.get_string("missing").is_none());
    }

    #[test]
    fn test_list_resource_bundle_put() {
        let mut bundle = I18nListResourceBundle::new("en_US");
        bundle.put("greeting", "Hello");
        assert_eq!(bundle.get_string("greeting"), Some("Hello"));
        assert_eq!(bundle.len(), 1);
    }

    #[test]
    fn test_list_resource_bundle_contains_key() {
        let bundle = I18nListResourceBundle::from_pairs(
            "ja_JP",
            vec![KeyValuePair::new("greeting", "こんにちは")],
        );
        assert!(bundle.contains_key("greeting"));
        assert!(!bundle.contains_key("missing"));
    }

    #[test]
    fn test_list_resource_bundle_keys() {
        let bundle = I18nListResourceBundle::from_pairs(
            "ko_KR",
            vec![
                KeyValuePair::new("a", "1"),
                KeyValuePair::new("b", "2"),
            ],
        );
        let keys = bundle.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("a"));
        assert!(keys.contains("b"));
    }

    #[test]
    fn test_list_resource_bundle_empty() {
        let bundle = I18nListResourceBundle::new("fr_FR");
        assert!(bundle.is_empty());
        assert_eq!(bundle.len(), 0);
    }
}
