//! 签名验证模块（对标 mybatis-plus-enhance `signature` 包）。
//!
//! 在 INSERT/UPDATE 时自动计算数据签名，在 SELECT 后自动验证签名完整性，
//! 防止数据被篡改。
//!
//! # 核心组件
//!
//! - [`DataSignatureHandler`] — 签名处理器 trait
//! - [`DefaultDataSignatureHandler`] — 默认 HMAC-SHA256 风格实现
//! - [`SignatureInnerInterceptor`] — 签名拦截器

pub mod handler;
pub mod default_handler;
pub mod interceptor;

pub use handler::DataSignatureHandler;
pub use default_handler::DefaultDataSignatureHandler;
pub use interceptor::SignatureInnerInterceptor;
