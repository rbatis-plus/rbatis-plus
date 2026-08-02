//! 资源包子系统（对标 MyBatis-Plus-Enhance `resource_bundle` 包）。
//!
//! 提供国际化翻译数据的加载和查找能力：
//! - `KeyValuePair` — 键值对基础结构
//! - `ResourceBundleEnumeration` — 键枚举器
//! - `EmptyResourceBundle` — 空资源包（fallback）
//! - `I18nListResourceBundle` — 基于列表的资源包
//! - `MultipleResourceBundle` — 多级资源包（parent chain 语义）

pub mod empty_resource_bundle;
pub mod i18n_list_resource_bundle;
pub mod key_value_pair;
pub mod multiple_resource_bundle;
pub mod resource_bundle_enumeration;

pub use empty_resource_bundle::EmptyResourceBundle;
pub use i18n_list_resource_bundle::I18nListResourceBundle;
pub use key_value_pair::KeyValuePair;
pub use multiple_resource_bundle::{MultipleResourceBundle, ResourceBundle};
pub use resource_bundle_enumeration::ResourceBundleEnumeration;
