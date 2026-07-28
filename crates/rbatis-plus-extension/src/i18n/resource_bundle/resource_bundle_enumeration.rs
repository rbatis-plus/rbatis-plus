//! 资源包枚举器（对标 Java `ResourceBundleEnumeration`）。
//!
//! 提供对资源包中所有键的迭代能力。

use std::collections::HashSet;

/// 资源包键枚举器。
///
/// 对应 Java：`java.util.ResourceBundle` 的内部枚举机制。
/// 在 Rust 中使用 `HashSet` 存储去重后的键集合，支持高效查找和遍历。
#[derive(Debug, Clone)]
pub struct ResourceBundleEnumeration {
    /// 去重后的键集合。
    keys: HashSet<String>,
}

impl ResourceBundleEnumeration {
    /// 从键列表创建枚举器。
    pub fn new(keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            keys: keys.into_iter().map(|k| k.into()).collect(),
        }
    }

    /// 获取键的总数。
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// 是否包含指定键。
    pub fn contains(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    /// 获取所有键的快照（有序不确定）。
    pub fn keys(&self) -> Vec<&str> {
        self.keys.iter().map(|s| s.as_str()).collect()
    }

    /// 转换为拥有所有权的键向量。
    pub fn into_keys(self) -> Vec<String> {
        self.keys.into_iter().collect()
    }
}

impl FromIterator<String> for ResourceBundleEnumeration {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self::new(iter)
    }
}

impl<'a> FromIterator<&'a str> for ResourceBundleEnumeration {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self::new(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumeration_creation() {
        let enum_keys = ResourceBundleEnumeration::new(vec!["greeting", "farewell"]);
        assert_eq!(enum_keys.len(), 2);
        assert!(enum_keys.contains("greeting"));
        assert!(enum_keys.contains("farewell"));
        assert!(!enum_keys.contains("missing"));
    }

    #[test]
    fn test_enumeration_dedup() {
        let enum_keys = ResourceBundleEnumeration::new(vec!["a", "b", "a", "c"]);
        assert_eq!(enum_keys.len(), 3);
    }

    #[test]
    fn test_enumeration_empty() {
        let enum_keys = ResourceBundleEnumeration::new(Vec::<&str>::new());
        assert!(enum_keys.is_empty());
        assert_eq!(enum_keys.len(), 0);
    }

    #[test]
    fn test_enumeration_from_iterator() {
        let keys = vec!["x".to_string(), "y".to_string()];
        let enum_keys: ResourceBundleEnumeration = keys.into_iter().collect();
        assert_eq!(enum_keys.len(), 2);
    }
}
