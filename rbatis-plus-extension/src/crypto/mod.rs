//! 加密模块（对标 mybatis-plus-enhance `crypto` 包）。
//!
//! 提供字段级加密/解密能力，在 INSERT/UPDATE 时自动加密，
//! SELECT 时自动解密，对业务代码完全透明。
//!
//! # 核心组件
//!
//! - [`EncryptedFieldHandler`] — 加密/解密/HMAC 处理器 trait
//! - [`DefaultEncryptedFieldHandler`] — 默认 XOR + Base64 实现
//! - [`CryptoInnerInterceptor`] — 加密拦截器

pub mod handler;
pub mod default_handler;
pub mod interceptor;

pub use handler::EncryptedFieldHandler;
pub use default_handler::DefaultEncryptedFieldHandler;
pub use interceptor::CryptoInnerInterceptor;
