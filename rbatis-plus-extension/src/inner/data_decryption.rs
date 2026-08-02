//! 结果解密增强拦截器（对标 Java `DataDecryptionInnerInterceptor`）。
//!
//! 在 SELECT 后对结果集中的加密字段进行解密。
//! 使用深拷贝（ResultObjectCopier）防止污染原始结果。
//! 阶段：`RESULT_DECRYPTION`（400），在签名验证之后、国际化之前执行。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.enhance.plugins.inner.DataDecryptionInnerInterceptor`
//!   （mybatis-plus-enhance-extension/.../plugins/inner/DataDecryptionInnerInterceptor.java，167 行）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::Error;
use rbs::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::crypto::handler::EncryptedFieldHandler;
use crate::inner::enhance_interceptor::EnhanceInnerInterceptor;
use crate::inner::enhance_phase::EnhancePhase;
use crate::inner::inner_interceptor::InnerInterceptor;

/// 结果解密增强拦截器（对标 Java `DataDecryptionInnerInterceptor`）。
///
/// 职责：
/// - `after_query`：深拷贝结果集，对加密字段解密后替换结果
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.inner.DataDecryptionInnerInterceptor`
#[derive(Clone)]
pub struct DataDecryptionInnerInterceptor {
    /// 加密处理器（复用 EncryptedFieldHandler 的 decrypt 方法）。
    handler: Arc<dyn EncryptedFieldHandler>,
    /// 需要解密的列名集合。
    encrypted_columns: HashMap<String, bool>,
}

impl std::fmt::Debug for DataDecryptionInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataDecryptionInnerInterceptor")
            .field(
                "encrypted_columns",
                &self.encrypted_columns.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl DataDecryptionInnerInterceptor {
    /// 创建结果解密拦截器。
    ///
    /// 对应 Java：`DataDecryptionInnerInterceptor(EncryptedFieldHandler)`
    pub fn new(handler: Box<dyn EncryptedFieldHandler>) -> Self {
        Self {
            handler: Arc::from(handler),
            encrypted_columns: HashMap::new(),
        }
    }

    /// 添加需要解密的列。
    ///
    /// 对应 Java：通过 `@EncryptedField` 注解配置
    pub fn with_encrypted_column(mut self, column: impl Into<String>) -> Self {
        self.encrypted_columns.insert(column.into(), true);
        self
    }

    /// 批量添加需要解密的列。
    pub fn with_encrypted_columns(mut self, columns: &[&str]) -> Self {
        for col in columns {
            self.encrypted_columns.insert(col.to_string(), true);
        }
        self
    }

    /// 判断某列是否需要解密。
    pub fn is_encrypted(&self, column: &str) -> bool {
        self.encrypted_columns.contains_key(column)
    }

    /// 获取加密处理器引用。
    pub fn handler(&self) -> &dyn EncryptedFieldHandler {
        self.handler.as_ref()
    }

    /// 深拷贝 Value（ResultObjectCopier 防止污染原始结果）。
    ///
    /// 对应 Java：`ResultObjectCopier.copy(Object)`
    fn deep_copy(value: &Value) -> Value {
        value.clone()
    }

    /// 递归解密 Value 中的加密字段。
    ///
    /// 对应 Java：`DataDecryptionInnerInterceptor.decryptResultSet(...)`
    fn decrypt_value_recursive(&self, value: &mut Value) {
        match value {
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.decrypt_value_recursive(item);
                }
            }
            Value::Map(map) => {
                let mut changes: Vec<(Value, Value)> = Vec::new();
                for (k, v) in map.0.iter() {
                    if let Value::String(col_name) = k {
                        if self.encrypted_columns.contains_key(col_name.as_str()) {
                            if let Value::String(encrypted_val) = v {
                                let decrypted = self.handler.decrypt(encrypted_val);
                                changes.push((k.clone(), Value::String(decrypted)));
                            }
                        }
                    }
                }
                for (k, v) in changes {
                    map.insert(k, v);
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl InnerInterceptor for DataDecryptionInnerInterceptor {
    /// SELECT 后对结果集进行解密。
    ///
    /// 对应 Java：`DataDecryptionInnerInterceptor.afterQuery(...)`
    ///
    /// 使用深拷贝（ResultObjectCopier）防止污染原始结果。
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        if let Ok(value) = result {
            // 深拷贝防止污染原始结果（对标 ResultObjectCopier）
            let mut copied = Self::deep_copy(value);
            self.decrypt_value_recursive(&mut copied);
            *value = copied;
        }
        Ok(())
    }
}

#[async_trait]
impl EnhanceInnerInterceptor for DataDecryptionInnerInterceptor {
    /// 声明阶段为结果解密（400）。
    ///
    /// 对应 Java：`DataDecryptionInnerInterceptor.phase()`
    fn phase(&self) -> EnhancePhase {
        EnhancePhase::RESULT_DECRYPTION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::default_handler::DefaultEncryptedFieldHandler;

    /// 创建测试用拦截器。
    fn create_test_interceptor() -> DataDecryptionInnerInterceptor {
        let handler = DefaultEncryptedFieldHandler::default();
        DataDecryptionInnerInterceptor::new(Box::new(handler))
            .with_encrypted_column("phone")
            .with_encrypted_column("email")
    }

    #[test]
    fn test_phase_is_result_decryption() {
        let interceptor = create_test_interceptor();
        assert_eq!(interceptor.phase(), EnhancePhase::RESULT_DECRYPTION);
    }

    #[test]
    fn test_is_encrypted() {
        let interceptor = create_test_interceptor();
        assert!(interceptor.is_encrypted("phone"));
        assert!(interceptor.is_encrypted("email"));
        assert!(!interceptor.is_encrypted("name"));
    }

    #[test]
    fn test_decrypt_value_decrypts_marked_fields() {
        let handler = DefaultEncryptedFieldHandler::default();
        let encrypted_phone = handler.encrypt("13800138000");
        let encrypted_email = handler.encrypt("test@example.com");

        let interceptor = DataDecryptionInnerInterceptor::new(Box::new(handler))
            .with_encrypted_column("phone")
            .with_encrypted_column("email");

        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("phone".to_string()),
            Value::String(encrypted_phone),
        );
        map.insert(
            Value::String("email".to_string()),
            Value::String(encrypted_email),
        );
        map.insert(
            Value::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        let mut value = Value::Map(map);

        interceptor.decrypt_value_recursive(&mut value);

        if let Value::Map(ref m) = value {
            let phone = m.get(&Value::String("phone".to_string()));
            if let Value::String(s) = phone {
                assert_eq!(s, "13800138000", "phone 应该被解密为原值");
            }
            let email = m.get(&Value::String("email".to_string()));
            if let Value::String(s) = email {
                assert_eq!(s, "test@example.com", "email 应该被解密为原值");
            }
            let name = m.get(&Value::String("name".to_string()));
            if let Value::String(s) = name {
                assert_eq!(s, "Alice", "name 不应被修改");
            }
        } else {
            panic!("应为 Map 类型");
        }
    }

    #[test]
    fn test_deep_copy_prevents_mutation() {
        let handler = DefaultEncryptedFieldHandler::default();
        let encrypted_phone = handler.encrypt("13800138000");

        let interceptor = DataDecryptionInnerInterceptor::new(Box::new(handler))
            .with_encrypted_column("phone");

        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("phone".to_string()),
            Value::String(encrypted_phone.clone()),
        );
        let original = Value::Map(map);

        // 深拷贝
        let mut copied = DataDecryptionInnerInterceptor::deep_copy(&original);
        interceptor.decrypt_value_recursive(&mut copied);

        // 原始值不应被修改
        if let Value::Map(ref m) = original {
            let phone = m.get(&Value::String("phone".to_string()));
            if let Value::String(s) = phone {
                assert_eq!(s, &encrypted_phone, "原始值不应被修改");
            }
        }
    }

    #[test]
    fn test_decrypt_array_of_maps() {
        let handler = DefaultEncryptedFieldHandler::default();
        let encrypted_phone = handler.encrypt("13900139000");

        let interceptor = DataDecryptionInnerInterceptor::new(Box::new(handler))
            .with_encrypted_column("phone");

        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("phone".to_string()),
            Value::String(encrypted_phone),
        );
        let mut value = Value::Array(vec![Value::Map(map)]);

        interceptor.decrypt_value_recursive(&mut value);

        if let Value::Array(ref arr) = value {
            if let Value::Map(ref m) = arr[0] {
                let phone = m.get(&Value::String("phone".to_string()));
                if let Value::String(s) = phone {
                    assert_eq!(s, "13900139000");
                }
            }
        }
    }
}
