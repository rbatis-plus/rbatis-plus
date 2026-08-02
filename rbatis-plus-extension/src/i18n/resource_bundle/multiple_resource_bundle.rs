//! 多级资源包（对标 Java `MultipleResourceBundle`）。
//!
//! 支持将多个资源包组合为一个，按优先级查找。
//! 优先级：第一个包最高（parent chain 语义），依次递减。

use super::empty_resource_bundle::EmptyResourceBundle;
use super::resource_bundle_enumeration::ResourceBundleEnumeration;

/// 资源包 trait（对标 Java `java.util.ResourceBundle`）。
///
/// 统一 `I18nListResourceBundle`、`EmptyResourceBundle` 和 `MultipleResourceBundle`
/// 的访问接口。
pub trait ResourceBundle: Send + Sync + std::fmt::Debug {
    /// 获取指定键的值。
    fn get_object(&self, key: &str) -> Option<&str>;

    /// 获取指定键的字符串值（与 `get_object` 等价）。
    fn get_string(&self, key: &str) -> Option<&str> {
        self.get_object(key)
    }

    /// 获取键枚举器。
    fn keys(&self) -> ResourceBundleEnumeration;

    /// 是否为空。
    fn is_empty(&self) -> bool {
        self.keys().is_empty()
    }

    /// 条目数量（近似值，合并包可能重复计数）。
    fn len(&self) -> usize {
        self.keys().len()
    }
}

/// 为 `EmptyResourceBundle` 实现 `ResourceBundle` trait。
impl ResourceBundle for EmptyResourceBundle {
    fn get_object(&self, _key: &str) -> Option<&str> {
        None
    }

    fn get_string(&self, _key: &str) -> Option<&str> {
        None
    }

    fn keys(&self) -> ResourceBundleEnumeration {
        ResourceBundleEnumeration::new(Vec::<&str>::new())
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn len(&self) -> usize {
        0
    }
}

/// 多级资源包，按优先级组合多个子资源包。
///
/// 对应 Java：MyBatis-Plus-Enhance 的 `MultipleResourceBundle`。
/// 查找时按顺序遍历子包，返回第一个命中的结果。
///
/// # 使用示例
///
/// ```ignore
/// let bundle = MultipleResourceBundle::new(vec![
///     Box::new(specific_bundle),  // 高优先级
///     Box::new(fallback_bundle),  // 低优先级
/// ]);
/// let value = bundle.get_string("greeting");
/// ```
#[derive(Debug)]
pub struct MultipleResourceBundle {
    /// 子资源包列表（按优先级排列）。
    bundles: Vec<Box<dyn ResourceBundle>>,
}

impl MultipleResourceBundle {
    /// 创建多级资源包。
    pub fn new(bundles: Vec<Box<dyn ResourceBundle>>) -> Self {
        Self { bundles }
    }

    /// 创建空的多级资源包。
    pub fn empty() -> Self {
        Self { bundles: Vec::new() }
    }

    /// 添加子资源包（追加到最低优先级）。
    pub fn add_bundle(&mut self, bundle: Box<dyn ResourceBundle>) {
        self.bundles.push(bundle);
    }

    /// 子包数量。
    pub fn bundle_count(&self) -> usize {
        self.bundles.len()
    }
}

impl ResourceBundle for MultipleResourceBundle {
    /// 按优先级查找键值：遍历子包，返回第一个命中的结果。
    fn get_object(&self, key: &str) -> Option<&str> {
        for bundle in &self.bundles {
            if let Some(value) = bundle.get_object(key) {
                return Some(value);
            }
        }
        None
    }

    /// 合并所有子包的键（去重由 `ResourceBundleEnumeration` 的 `HashSet` 保证）。
    fn keys(&self) -> ResourceBundleEnumeration {
        let all_keys: Vec<String> = self
            .bundles
            .iter()
            .flat_map(|b| b.keys().into_keys())
            .collect();
        ResourceBundleEnumeration::new(all_keys)
    }

    fn is_empty(&self) -> bool {
        self.bundles.iter().all(|b| b.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::resource_bundle::i18n_list_resource_bundle::I18nListResourceBundle;
    use crate::i18n::resource_bundle::key_value_pair::KeyValuePair;

    #[test]
    fn test_multiple_resource_bundle_priority() {
        let specific = I18nListResourceBundle::from_pairs(
            "zh_CN",
            vec![
                KeyValuePair::new("greeting", "你好（具体）"),
                KeyValuePair::new("specific_only", "仅具体包"),
            ],
        );
        let fallback = I18nListResourceBundle::from_pairs(
            "zh_CN",
            vec![
                KeyValuePair::new("greeting", "你好（通用）"),
                KeyValuePair::new("fallback_only", "仅通用包"),
            ],
        );

        let bundle = MultipleResourceBundle::new(vec![
            Box::new(specific),
            Box::new(fallback),
        ]);

        // 高优先级包的值应被返回
        assert_eq!(bundle.get_object("greeting"), Some("你好（具体）"));
        // 各自独有的键也应可访问
        assert_eq!(bundle.get_object("specific_only"), Some("仅具体包"));
        assert_eq!(bundle.get_object("fallback_only"), Some("仅通用包"));
        // 不存在的键
        assert!(bundle.get_object("missing").is_none());
    }

    #[test]
    fn test_multiple_resource_bundle_with_empty() {
        let empty = EmptyResourceBundle;
        let bundle = MultipleResourceBundle::new(vec![Box::new(empty)]);
        assert!(bundle.get_object("any").is_none());
        assert!(bundle.is_empty());
    }

    #[test]
    fn test_multiple_resource_bundle_keys_merged() {
        let a = I18nListResourceBundle::from_pairs(
            "en",
            vec![KeyValuePair::new("k1", "v1")],
        );
        let b = I18nListResourceBundle::from_pairs(
            "en",
            vec![KeyValuePair::new("k2", "v2")],
        );
        let bundle = MultipleResourceBundle::new(vec![Box::new(a), Box::new(b)]);
        let keys = bundle.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains("k1"));
        assert!(keys.contains("k2"));
    }
}
