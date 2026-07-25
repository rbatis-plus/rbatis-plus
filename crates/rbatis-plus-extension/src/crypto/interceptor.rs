//! 加密拦截器（对标 mybatis-plus-enhance `EncryptedFieldInnerInterceptor`）。
//!
//! 在 INSERT/UPDATE 时自动加密标记字段，在 SELECT 时自动解密。

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbs::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::handler::EncryptedFieldHandler;

/// 加密字段拦截器（对标 Java `EncryptedFieldInnerInterceptor`）。
///
/// 拦截 SELECT 操作，对标记了加密属性的字段自动解密。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.crypto.EncryptedFieldInnerInterceptor`
///
/// # 用法
///
/// ```ignore
/// use rbatis_plus_extension::crypto::{CryptoInnerInterceptor, DefaultEncryptedFieldHandler};
///
/// let handler = DefaultEncryptedFieldHandler::default();
/// let interceptor = CryptoInnerInterceptor::new(Box::new(handler))
///     .with_encrypted_column("name")
///     .with_encrypted_column("email");
/// ```
pub struct CryptoInnerInterceptor {
    /// 加密处理器。
    handler: Arc<dyn EncryptedFieldHandler>,
    /// 需要加密的列名集合。
    encrypted_columns: HashMap<String, bool>,
}

impl std::fmt::Debug for CryptoInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryptoInnerInterceptor")
            .field("encrypted_columns", &self.encrypted_columns.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl CryptoInnerInterceptor {
    /// 创建加密拦截器。
    pub fn new(handler: Box<dyn EncryptedFieldHandler>) -> Self {
        Self {
            handler: Arc::from(handler),
            encrypted_columns: HashMap::new(),
        }
    }

    /// 添加需要加密的列。
    pub fn with_encrypted_column(mut self, column: impl Into<String>) -> Self {
        self.encrypted_columns.insert(column.into(), true);
        self
    }

    /// 批量添加需要加密的列。
    pub fn with_encrypted_columns(mut self, columns: &[&str]) -> Self {
        for col in columns {
            self.encrypted_columns.insert(col.to_string(), true);
        }
        self
    }

    /// 判断某列是否需要加密。
    pub fn is_encrypted(&self, column: &str) -> bool {
        self.encrypted_columns.contains_key(column)
    }

    /// 对值进行加密。
    pub fn encrypt_value(&self, value: &str) -> String {
        self.handler.encrypt(value)
    }

    /// 对值进行解密。
    pub fn decrypt_value(&self, encrypted: &str) -> String {
        self.handler.decrypt(encrypted)
    }

    /// 获取处理器引用。
    pub fn handler(&self) -> &dyn EncryptedFieldHandler {
        self.handler.as_ref()
    }
}

#[async_trait]
impl InnerInterceptor for CryptoInnerInterceptor {
    /// SELECT 后解密结果集中的加密字段。
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        if let Ok(value) = result {
            decrypt_value_recursive(value, &self.handler, &self.encrypted_columns);
        }
        Ok(())
    }
}

/// 递归解密 Value 中的加密字段。
///
/// 对标 `EncryptedFieldInnerInterceptor.decryptResultSet()`
fn decrypt_value_recursive(
    value: &mut Value,
    handler: &Arc<dyn EncryptedFieldHandler>,
    encrypted_columns: &HashMap<String, bool>,
) {
    match value {
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                decrypt_value_recursive(item, handler, encrypted_columns);
            }
        }
        Value::Map(map) => {
            // 收集需要解密的 (key, decrypted_value) 对
            let mut changes: Vec<(Value, Value)> = Vec::new();
            for (k, v) in map.0.iter() {
                if let Value::String(col_name) = k {
                    if encrypted_columns.contains_key(col_name.as_str()) {
                        if let Value::String(encrypted_val) = v {
                            let decrypted = handler.decrypt(encrypted_val);
                            changes.push((k.clone(), Value::String(decrypted)));
                        }
                    }
                }
            }
            // 应用解密后的值
            for (k, v) in changes {
                map.insert(k, v);
            }
        }
        _ => {}
    }
}

/// 对 INSERT/UPDATE 参数中的加密字段进行加密。
///
/// 在 `before_update` 中调用，对标 `EncryptedFieldInnerInterceptor.encryptArgs()`
pub fn encrypt_args(
    args: &mut [Value],
    handler: &dyn EncryptedFieldHandler,
    encrypted_columns: &HashMap<String, bool>,
) {
    for arg in args.iter_mut() {
        encrypt_value_recursive(arg, handler, encrypted_columns);
    }
}

/// 递归加密 Value 中的加密字段。
fn encrypt_value_recursive(
    value: &mut Value,
    handler: &dyn EncryptedFieldHandler,
    encrypted_columns: &HashMap<String, bool>,
) {
    match value {
        Value::Map(map) => {
            let mut changes: Vec<(Value, Value)> = Vec::new();
            for (k, v) in map.0.iter() {
                if let Value::String(col_name) = k {
                    if encrypted_columns.contains_key(col_name.as_str()) {
                        if let Value::String(plain_val) = v {
                            let encrypted = handler.encrypt(plain_val);
                            changes.push((k.clone(), Value::String(encrypted)));
                        }
                    }
                }
            }
            for (k, v) in changes {
                map.insert(k, v);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                encrypt_value_recursive(item, handler, encrypted_columns);
            }
        }
        _ => {}
    }
}
