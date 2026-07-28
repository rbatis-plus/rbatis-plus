//! 参数加密增强拦截器（对标 Java `DataEncryptionInnerInterceptor`）。
//!
//! 在 INSERT/UPDATE/SELECT 操作前，对参数中的加密字段进行加密处理。
//! 阶段：`PARAMETER_ENCRYPTION`（200），在签名之前执行。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.enhance.plugins.inner.DataEncryptionInnerInterceptor`
//!   （mybatis-plus-enhance-extension/.../plugins/inner/DataEncryptionInnerInterceptor.java，215 行）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::intercept::Action;
use rbatis::Error;
use rbdc::db::ExecResult;
use rbs::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::crypto::handler::EncryptedFieldHandler;
use crate::inner::enhance_interceptor::EnhanceInnerInterceptor;
use crate::inner::enhance_phase::EnhancePhase;
use crate::inner::inner_interceptor::InnerInterceptor;

/// 参数加密增强拦截器（对标 Java `DataEncryptionInnerInterceptor`）。
///
/// 职责：
/// - `before_update`：遍历 INSERT/UPDATE 参数，加密标记字段
/// - `before_query`：加密查询条件中的加密字段（如手机号模糊匹配）
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.inner.DataEncryptionInnerInterceptor`
#[derive(Clone)]
pub struct DataEncryptionInnerInterceptor {
    /// 加密处理器。
    handler: Arc<dyn EncryptedFieldHandler>,
    /// 需要加密的列名集合。
    encrypted_columns: HashMap<String, bool>,
}

impl std::fmt::Debug for DataEncryptionInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataEncryptionInnerInterceptor")
            .field(
                "encrypted_columns",
                &self.encrypted_columns.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl DataEncryptionInnerInterceptor {
    /// 创建参数加密拦截器。
    ///
    /// 对应 Java：`DataEncryptionInnerInterceptor(EncryptedFieldHandler)`
    pub fn new(handler: Box<dyn EncryptedFieldHandler>) -> Self {
        Self {
            handler: Arc::from(handler),
            encrypted_columns: HashMap::new(),
        }
    }

    /// 添加需要加密的列。
    ///
    /// 对应 Java：通过 `@EncryptedField` 注解配置
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

    /// 获取加密处理器引用。
    pub fn handler(&self) -> &dyn EncryptedFieldHandler {
        self.handler.as_ref()
    }

    /// 对参数中的加密字段进行加密。
    fn encrypt_args(&self, args: &mut [Value]) {
        for arg in args.iter_mut() {
            self.encrypt_value_recursive(arg);
        }
    }

    /// 递归加密 Value 中的加密字段。
    fn encrypt_value_recursive(&self, value: &mut Value) {
        match value {
            Value::Map(map) => {
                let mut changes: Vec<(Value, Value)> = Vec::new();
                for (k, v) in map.0.iter() {
                    if let Value::String(col_name) = k {
                        if self.encrypted_columns.contains_key(col_name.as_str()) {
                            if let Value::String(plain_val) = v {
                                let encrypted = self.handler.encrypt(plain_val);
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
                    self.encrypt_value_recursive(item);
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl InnerInterceptor for DataEncryptionInnerInterceptor {
    /// INSERT/UPDATE 前加密参数。
    ///
    /// 对应 Java：`DataEncryptionInnerInterceptor.beforeUpdate(...)`
    async fn before_update(
        &self,
        _executor: &dyn Executor,
        _sql: &mut String,
        args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        self.encrypt_args(args);
        Ok(Action::Next)
    }

    /// SELECT 前加密查询条件中的加密字段。
    ///
    /// 对应 Java：`DataEncryptionInnerInterceptor.beforeQuery(...)`
    /// 用于支持加密字段的等值查询（如手机号匹配）。
    async fn before_query(
        &self,
        _executor: &dyn Executor,
        _sql: &mut String,
        args: &mut Vec<Value>,
        _result: &mut Result<Value, Error>,
    ) -> Result<Action, Error> {
        self.encrypt_args(args);
        Ok(Action::Next)
    }
}

#[async_trait]
impl EnhanceInnerInterceptor for DataEncryptionInnerInterceptor {
    /// 声明阶段为参数加密（200）。
    ///
    /// 对应 Java：`DataEncryptionInnerInterceptor.phase()`
    fn phase(&self) -> EnhancePhase {
        EnhancePhase::PARAMETER_ENCRYPTION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::default_handler::DefaultEncryptedFieldHandler;

    /// 创建测试用拦截器。
    fn create_test_interceptor() -> DataEncryptionInnerInterceptor {
        let handler = DefaultEncryptedFieldHandler::default();
        DataEncryptionInnerInterceptor::new(Box::new(handler))
            .with_encrypted_column("phone")
            .with_encrypted_column("email")
    }

    #[test]
    fn test_phase_is_parameter_encryption() {
        let interceptor = create_test_interceptor();
        assert_eq!(interceptor.phase(), EnhancePhase::PARAMETER_ENCRYPTION);
    }

    #[test]
    fn test_is_encrypted() {
        let interceptor = create_test_interceptor();
        assert!(interceptor.is_encrypted("phone"));
        assert!(interceptor.is_encrypted("email"));
        assert!(!interceptor.is_encrypted("name"));
    }

    #[test]
    fn test_encrypt_args_encrypts_marked_fields() {
        let interceptor = create_test_interceptor();
        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("phone".to_string()),
            Value::String("13800138000".to_string()),
        );
        map.insert(
            Value::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        let mut args = vec![Value::Map(map)];

        interceptor.encrypt_args(&mut args);

        if let Value::Map(ref m) = args[0] {
            // phone 应该被加密（不再是原值）
            let phone = m.get(&Value::String("phone".to_string()));
            if let Value::String(s) = phone {
                assert_ne!(s, "13800138000", "phone 应该被加密");
            } else {
                panic!("phone 字段应为 String 类型");
            }
            // name 不应被加密
            let name = m.get(&Value::String("name".to_string()));
            if let Value::String(s) = name {
                assert_eq!(s, "Alice", "name 不应被加密");
            }
        } else {
            panic!("参数应为 Map 类型");
        }
    }

    #[test]
    fn test_encrypt_args_handles_array() {
        let interceptor = create_test_interceptor();
        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("phone".to_string()),
            Value::String("13900139000".to_string()),
        );
        let mut args = vec![Value::Array(vec![Value::Map(map)])];

        interceptor.encrypt_args(&mut args);

        if let Value::Array(ref arr) = args[0] {
            if let Value::Map(ref m) = arr[0] {
                let phone = m.get(&Value::String("phone".to_string()));
                if let Value::String(s) = phone {
                    assert_ne!(s, "13900139000");
                }
            }
        }
    }

    #[test]
    fn test_with_encrypted_columns_batch() {
        let handler = DefaultEncryptedFieldHandler::default();
        let interceptor =
            DataEncryptionInnerInterceptor::new(Box::new(handler)).with_encrypted_columns(&["a", "b", "c"]);
        assert!(interceptor.is_encrypted("a"));
        assert!(interceptor.is_encrypted("b"));
        assert!(interceptor.is_encrypted("c"));
        assert!(!interceptor.is_encrypted("d"));
    }
}
