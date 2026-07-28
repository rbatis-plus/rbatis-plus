//! 空资源包（对标 Java `EmptyResourceBundle`）。
//!
//! 当指定语言环境的资源包不存在时，返回空资源包而非 null，
//! 避免空指针异常。

use super::resource_bundle_enumeration::ResourceBundleEnumeration;

/// 空资源包，不包含任何国际化条目。
///
/// 对应 Java：MyBatis-Plus-Enhance 的 `EmptyResourceBundle`。
/// 用作 fallback，避免 `Option` 层层嵌套。
#[derive(Debug, Clone, Copy)]
pub struct EmptyResourceBundle;

impl EmptyResourceBundle {
    /// 获取指定键的值（始终返回 `None`）。
    pub fn get_object(&self, _key: &str) -> Option<&str> {
        None
    }

    /// 获取指定键的字符串值（始终返回空字符串）。
    pub fn get_string(&self, _key: &str) -> &str {
        ""
    }

    /// 获取键枚举器（始终为空）。
    pub fn keys(&self) -> ResourceBundleEnumeration {
        ResourceBundleEnumeration::new(Vec::<&str>::new())
    }

    /// 是否为空资源包（始终为 `true`）。
    pub fn is_empty(&self) -> bool {
        true
    }

    /// 条目数量（始终为 0）。
    pub fn len(&self) -> usize {
        0
    }
}

impl Default for EmptyResourceBundle {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_resource_bundle_get_object() {
        let bundle = EmptyResourceBundle;
        assert!(bundle.get_object("any_key").is_none());
    }

    #[test]
    fn test_empty_resource_bundle_get_string() {
        let bundle = EmptyResourceBundle;
        assert_eq!(bundle.get_string("any_key"), "");
    }

    #[test]
    fn test_empty_resource_bundle_keys() {
        let bundle = EmptyResourceBundle;
        let keys = bundle.keys();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_empty_resource_bundle_is_empty() {
        let bundle = EmptyResourceBundle;
        assert!(bundle.is_empty());
        assert_eq!(bundle.len(), 0);
    }
}
