//! 国际化模块（对标 mybatis-plus-enhance `i18n` 包）。
//!
//! 提供查询结果国际化翻译支持：
//! - [`context`] — 线程局部 Locale 管理
//! - [`handler`] — 国际化处理器 trait 和默认实现
//! - [`resource_bundle`] — 资源包子系统

pub mod context;
pub mod handler;
pub mod resource_bundle;

pub use context::I18nContext;
pub use handler::{DataI18nHandler, DefaultDataI18nHandler, NoopDataI18nHandler};
pub use resource_bundle::{
    EmptyResourceBundle, I18nListResourceBundle, KeyValuePair,
    MultipleResourceBundle, ResourceBundle, ResourceBundleEnumeration,
};
