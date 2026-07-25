#[derive(Debug, Clone)]
pub struct I18nColumnAttr {
    pub column: &'static str,
    pub key: &'static str,
}

impl Default for I18nColumnAttr {
    fn default() -> Self {
        Self { column: "", key: "" }
    }
}

pub trait I18nColumn {
    fn i18n_columns() -> Vec<I18nColumnAttr>;
}
