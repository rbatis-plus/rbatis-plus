//! 国际化处理器（对标 Java `DataI18nHandler` / `DefaultDataI18nHandler`）。
//!
//! 定义结果集翻译的核心 trait 和默认实现。
//! `DataI18nHandler::handle` 在 `DataI18nInnerInterceptor::after_query` 中被调用，
//! 遍历结果集并根据当前 Locale 替换国际化列的值。

use std::collections::HashMap;

use rbs::Value;

use super::resource_bundle::i18n_list_resource_bundle::I18nListResourceBundle;
use super::resource_bundle::key_value_pair::KeyValuePair;

/// 国际化处理器 trait（对标 Java `DataI18nHandler`）。
///
/// 定义结果集翻译的抽象接口。用户可实现此 trait 以自定义翻译逻辑
///（如从 Redis、数据库或其他数据源加载翻译数据）。
pub trait DataI18nHandler: Send + Sync + std::fmt::Debug {
    /// 翻译结果集中的国际化字段。
    ///
    /// # 参数
    ///
    /// - `locale` — 当前语言环境（如 "zh_CN"、"en_US"）
    /// - `ms_id` — Mapper 语句 ID（如 "UserMapper.selectById"），用于
    ///   判断是否需要翻译
    /// - `results` — 查询结果集（可变引用，就地修改）
    ///
    /// # 对应 Java
    ///
    /// `DataI18nHandler.handle(String locale, String ms, List<Object> results)`
    fn handle(
        &self,
        locale: &str,
        ms_id: &str,
        results: &mut Value,
    ) -> Result<(), rbatis::Error>;
}

/// 默认国际化处理器（对标 Java `DefaultDataI18nHandler`）。
///
/// 从 `ResourceBundle` 加载翻译数据，遍历结果集中的 Map 行，
/// 对匹配的列进行值替换。
///
/// # 翻译策略
///
/// 对结果集中的每一行（Map 类型）：
/// 1. 获取 `_i18n` 字段的值作为翻译键（如 "user.name"）
/// 2. 从 ResourceBundle 中查找 `{locale}.{key}` 对应的翻译
/// 3. 替换目标列的值
///
/// 若结果行为数组，则递归处理每个元素。
///
/// # 对应 Java
///
/// `com.baomidou.mybatisplus.enhance.plugins.inner.DefaultDataI18nHandler`
#[derive(Debug, Clone)]
pub struct DefaultDataI18nHandler {
    /// 语言环境 -> 翻译键 -> 翻译值 的映射。
    ///
    /// 对应 Java：`Map<String, I18nListResourceBundle> localeBundlesMap`
    locale_bundles: HashMap<String, I18nListResourceBundle>,
    /// 国际化列名（存放翻译键的列名，默认 "_i18n"）。
    ///
    /// 对应 Java：`@I18nColumn.value()` 注解
    i18n_column: String,
    /// 需要翻译的目标列名列表。
    ///
    /// 对应 Java：通过 `@I18nColumn` 注解的字段列表
    target_columns: Vec<String>,
}

impl DefaultDataI18nHandler {
    /// 创建默认国际化处理器。
    pub fn new() -> Self {
        Self {
            locale_bundles: HashMap::new(),
            i18n_column: "_i18n".to_string(),
            target_columns: Vec::new(),
        }
    }

    /// 设置国际化列名（存放翻译键的列名）。
    ///
    /// 对应 Java：`@I18nColumn` 注解的 `value` 属性
    pub fn with_i18n_column(mut self, column: impl Into<String>) -> Self {
        self.i18n_column = column.into();
        self
    }

    /// 添加需要翻译的目标列。
    pub fn with_target_column(mut self, column: impl Into<String>) -> Self {
        self.target_columns.push(column.into());
        self
    }

    /// 批量添加需要翻译的目标列。
    pub fn with_target_columns(mut self, columns: &[&str]) -> Self {
        for col in columns {
            self.target_columns.push(col.to_string());
        }
        self
    }

    /// 添加语言环境的翻译数据。
    ///
    /// 对应 Java：`DefaultDataI18nHandler.addResourceBundle(locale, pairs)`
    pub fn add_resource_bundle(
        &mut self,
        locale: impl Into<String>,
        pairs: Vec<KeyValuePair>,
    ) {
        let locale = locale.into();
        let bundle = I18nListResourceBundle::from_pairs(&locale, pairs);
        self.locale_bundles.insert(locale, bundle);
    }

    /// 获取指定语言环境的资源包。
    pub fn get_bundle(&self, locale: &str) -> Option<&I18nListResourceBundle> {
        self.locale_bundles.get(locale)
    }

    /// 获取国际化列名。
    pub fn i18n_column(&self) -> &str {
        &self.i18n_column
    }

    /// 获取目标列名列表。
    pub fn target_columns(&self) -> &[String] {
        &self.target_columns
    }

    /// 翻译单行结果（Map 类型）。
    ///
    /// 对应 Java：`DefaultDataI18nHandler.translateRow(...)`
    fn translate_row(
        &self,
        locale: &str,
        map: &mut rbs::value::map::ValueMap,
    ) {
        // 获取翻译键
        let i18n_key = match map.get(&Value::String(self.i18n_column.clone())) {
            Value::String(s) => s.to_string(),
            _ => return, // 没有国际化列，跳过
        };

        // 获取资源包
        let bundle = match self.locale_bundles.get(locale) {
            Some(b) => b,
            None => return, // 没有该语言环境的翻译数据
        };

        // 翻译每个目标列
        for col in &self.target_columns {
            let lookup_key = format!("{}.{}", i18n_key, col);
            if let Some(translated) = bundle.get_string(&lookup_key) {
                let col_key = Value::String(col.clone());
                // 只在翻译值非空时替换
                if !translated.is_empty() {
                    map.insert(col_key, Value::String(translated.to_string()));
                }
            }
        }
    }

    /// 递归翻译结果集。
    ///
    /// 对应 Java：`DefaultDataI18nHandler.handle(...)`
    fn translate_recursive(&self, locale: &str, value: &mut Value) {
        match value {
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.translate_recursive(locale, item);
                }
            }
            Value::Map(map) => {
                self.translate_row(locale, map);
            }
            _ => {}
        }
    }
}

impl Default for DefaultDataI18nHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DataI18nHandler for DefaultDataI18nHandler {
    /// 翻译结果集中的国际化字段。
    ///
    /// 对应 Java：`DefaultDataI18nHandler.handle(String locale, String ms, List<Object> results)`
    fn handle(
        &self,
        locale: &str,
        _ms_id: &str,
        results: &mut Value,
    ) -> Result<(), rbatis::Error> {
        self.translate_recursive(locale, results);
        Ok(())
    }
}

/// 空操作国际化处理器（不执行任何翻译）。
///
/// 用于不需要翻译但需要注册拦截器的场景。
#[derive(Debug, Clone, Copy)]
pub struct NoopDataI18nHandler;

impl DataI18nHandler for NoopDataI18nHandler {
    fn handle(
        &self,
        _locale: &str,
        _ms_id: &str,
        _results: &mut Value,
    ) -> Result<(), rbatis::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbs::value::map::ValueMap;

    /// 创建带翻译数据的测试处理器。
    fn create_test_handler() -> DefaultDataI18nHandler {
        let mut handler = DefaultDataI18nHandler::new()
            .with_i18n_column("_i18n")
            .with_target_columns(&["name", "desc"]);

        handler.add_resource_bundle(
            "zh_CN",
            vec![
                KeyValuePair::new("product.phone.name", "手机"),
                KeyValuePair::new("product.phone.desc", "智能手机"),
                KeyValuePair::new("product.laptop.name", "笔记本电脑"),
                KeyValuePair::new("product.laptop.desc", "便携式电脑"),
            ],
        );

        handler.add_resource_bundle(
            "en_US",
            vec![
                KeyValuePair::new("product.phone.name", "Phone"),
                KeyValuePair::new("product.phone.desc", "Smart Phone"),
                KeyValuePair::new("product.laptop.name", "Laptop"),
                KeyValuePair::new("product.laptop.desc", "Portable Computer"),
            ],
        );

        handler
    }

    #[test]
    fn test_handler_translate_zh_cn() {
        let handler = create_test_handler();

        let mut map = ValueMap::new();
        map.insert(Value::String("_i18n".into()), Value::String("product.phone".into()));
        map.insert(Value::String("name".into()), Value::String("Phone".into()));
        map.insert(Value::String("desc".into()), Value::String("Smart Phone".into()));
        map.insert(Value::String("price".into()), Value::I64(999));

        let mut value = Value::Map(map);
        handler.handle("zh_CN", "test", &mut value).unwrap();

        if let Value::Map(ref m) = value {
            let name = m.get(&Value::String("name".into()));
            if let Value::String(s) = name {
                assert_eq!(s, "手机", "name 应翻译为中文");
            }
            let desc = m.get(&Value::String("desc".into()));
            if let Value::String(s) = desc {
                assert_eq!(s, "智能手机", "desc 应翻译为中文");
            }
            let price = m.get(&Value::String("price".into()));
            if let Value::I64(n) = price {
                assert_eq!(*n, 999, "price 不应被翻译");
            }
        }
    }

    #[test]
    fn test_handler_translate_en_us() {
        let handler = create_test_handler();

        let mut map = ValueMap::new();
        map.insert(Value::String("_i18n".into()), Value::String("product.phone".into()));
        map.insert(Value::String("name".into()), Value::String("手机".into()));

        let mut value = Value::Map(map);
        handler.handle("en_US", "test", &mut value).unwrap();

        if let Value::Map(ref m) = value {
            let name = m.get(&Value::String("name".into()));
            if let Value::String(s) = name {
                assert_eq!(s, "Phone");
            }
        }
    }

    #[test]
    fn test_handler_translate_array() {
        let handler = create_test_handler();

        let mut map1 = ValueMap::new();
        map1.insert(Value::String("_i18n".into()), Value::String("product.phone".into()));
        map1.insert(Value::String("name".into()), Value::String("Phone".into()));

        let mut map2 = ValueMap::new();
        map2.insert(Value::String("_i18n".into()), Value::String("product.laptop".into()));
        map2.insert(Value::String("name".into()), Value::String("Laptop".into()));

        let mut value = Value::Array(vec![Value::Map(map1), Value::Map(map2)]);
        handler.handle("zh_CN", "test", &mut value).unwrap();

        if let Value::Array(ref arr) = value {
            if let Value::Map(ref m1) = arr[0] {
                let name = m1.get(&Value::String("name".into()));
                if let Value::String(s) = name {
                    assert_eq!(s, "手机");
                }
            }
            if let Value::Map(ref m2) = arr[1] {
                let name = m2.get(&Value::String("name".into()));
                if let Value::String(s) = name {
                    assert_eq!(s, "笔记本电脑");
                }
            }
        }
    }

    #[test]
    fn test_handler_no_i18n_column_skips() {
        let handler = create_test_handler();

        let mut map = ValueMap::new();
        map.insert(Value::String("name".into()), Value::String("Phone".into()));
        // 没有 _i18n 列

        let mut value = Value::Map(map);
        handler.handle("zh_CN", "test", &mut value).unwrap();

        if let Value::Map(ref m) = value {
            let name = m.get(&Value::String("name".into()));
            if let Value::String(s) = name {
                assert_eq!(s, "Phone", "没有 _i18n 列时不应翻译");
            }
        }
    }

    #[test]
    fn test_handler_missing_locale_no_panic() {
        let handler = create_test_handler();

        let mut map = ValueMap::new();
        map.insert(Value::String("_i18n".into()), Value::String("product.phone".into()));
        map.insert(Value::String("name".into()), Value::String("Phone".into()));

        let mut value = Value::Map(map);
        // 不存在的语言环境不应 panic
        handler.handle("fr_FR", "test", &mut value).unwrap();

        if let Value::Map(ref m) = value {
            let name = m.get(&Value::String("name".into()));
            if let Value::String(s) = name {
                assert_eq!(s, "Phone", "不存在的语言环境不应修改值");
            }
        }
    }

    #[test]
    fn test_handler_missing_key_no_change() {
        let handler = create_test_handler();

        let mut map = ValueMap::new();
        map.insert(Value::String("_i18n".into()), Value::String("product.tablet".into()));
        map.insert(Value::String("name".into()), Value::String("Tablet".into()));

        let mut value = Value::Map(map);
        handler.handle("zh_CN", "test", &mut value).unwrap();

        if let Value::Map(ref m) = value {
            let name = m.get(&Value::String("name".into()));
            if let Value::String(s) = name {
                assert_eq!(s, "Tablet", "不存在的翻译键不应修改值");
            }
        }
    }
}
