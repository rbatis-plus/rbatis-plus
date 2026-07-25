//! 国际化模块（对标 mybatis-plus-enhance `i18n` 包）。
//!
//! 提供国际化列支持，在 SELECT 查询时根据当前语言环境自动选择对应的列。

/// 国际化处理器 trait（对标 Java `I18nHandler`）。
pub trait I18nHandler: Send + Sync + 'static {
    /// 获取当前语言环境（如 "zh_CN"、"en_US"）。
    fn current_locale(&self) -> &str;

    /// 根据语言环境获取对应的列名。
    ///
    /// 例如：`column = "name"` + `locale = "zh_CN"` → `"name_zh_CN"`
    fn resolve_column(&self, column: &str, locale: &str) -> String {
        format!("{}_{}", column, locale)
    }
}

/// 默认国际化处理器（使用固定语言环境）。
#[derive(Debug, Clone)]
pub struct DefaultI18nHandler {
    locale: String,
}

impl Default for DefaultI18nHandler {
    fn default() -> Self {
        Self { locale: "zh_CN".to_string() }
    }
}

impl DefaultI18nHandler {
    pub fn new(locale: impl Into<String>) -> Self {
        Self { locale: locale.into() }
    }
}

impl I18nHandler for DefaultI18nHandler {
    fn current_locale(&self) -> &str {
        &self.locale
    }
}
