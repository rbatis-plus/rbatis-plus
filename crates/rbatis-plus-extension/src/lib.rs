//! Extension crate: interceptors, service layer, and enhancement plugins.
//!
//! # 增强模块（对标 mybatis-plus-enhance）
//!
//! - [`crypto`] — 字段加密/解密
//! - [`signature`] — 数据签名验证
//! - [`i18n`] — 国际化列支持
//! - [`observation`] — SQL 执行观察
//! - [`insert_ignore`] — INSERT IGNORE 支持

pub mod crypto;
pub mod i18n;
pub mod inner;
pub mod insert_ignore;
pub mod observation;
pub mod service;
pub mod signature;

// 复用上游 rbatis 主仓的缓存实现
pub use rbatis::plugin::cache::*;
