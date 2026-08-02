//! 数据签名增强拦截器（对标 Java `DataSignatureInnerInterceptor`）。
//!
//! 在 INSERT/UPDATE 时自动计算数据签名，在 SELECT 后验证签名完整性。
//! 阶段：`DATA_SIGNATURE`（300），在参数加密之后、结果解密之前执行。
//!
//! # 对应 Java
//!
//! - `com.baomidou.mybatisplus.enhance.plugins.inner.DataSignatureInnerInterceptor`
//!   （mybatis-plus-enhance-extension/.../plugins/inner/DataSignatureInnerInterceptor.java，253 行）

use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::intercept::Action;
use rbatis::Error;
use rbdc::db::ExecResult;
use rbs::Value;
use std::sync::Arc;

use crate::inner::enhance_interceptor::EnhanceInnerInterceptor;
use crate::inner::enhance_phase::EnhancePhase;
use crate::inner::inner_interceptor::InnerInterceptor;
use crate::signature::handler::DataSignatureHandler;

/// 数据签名增强拦截器（对标 Java `DataSignatureInnerInterceptor`）。
///
/// 职责：
/// - `before_update`：INSERT/UPDATE 前计算数据签名并添加签名列
/// - `after_query`：SELECT 后验证结果集中的签名完整性
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.plugins.inner.DataSignatureInnerInterceptor`
#[derive(Clone)]
pub struct DataSignatureInnerInterceptor {
    /// 签名处理器。
    handler: Arc<dyn DataSignatureHandler>,
}

impl std::fmt::Debug for DataSignatureInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataSignatureInnerInterceptor")
            .field("signature_column", &self.handler.signature_column())
            .finish()
    }
}

impl DataSignatureInnerInterceptor {
    /// 创建数据签名拦截器。
    ///
    /// 对应 Java：`DataSignatureInnerInterceptor(DataSignatureHandler)`
    pub fn new(handler: Box<dyn DataSignatureHandler>) -> Self {
        Self {
            handler: Arc::from(handler),
        }
    }

    /// 获取签名处理器引用。
    pub fn handler(&self) -> &dyn DataSignatureHandler {
        self.handler.as_ref()
    }

    /// 计算数据签名。
    pub fn sign(&self, data: &str) -> String {
        self.handler.sign(data)
    }

    /// 验证数据签名。
    pub fn verify(&self, data: &str, signature: &str) -> bool {
        self.handler.verify(data, signature)
    }

    /// 对 INSERT/UPDATE 参数计算签名并添加签名列。
    ///
    /// 对应 Java：`DataSignatureInnerInterceptor.signArgs(...)`
    fn sign_args(&self, args: &mut [Value]) {
        let sig_col = self.handler.signature_column();
        for arg in args.iter_mut() {
            if let Value::Map(map) = arg {
                let sign_data = self.build_sign_data(map, sig_col);
                let signature = self.handler.sign(&sign_data);
                map.insert(
                    Value::String(sig_col.to_string()),
                    Value::String(signature),
                );
            }
        }
    }

    /// 验证结果集中的签名。
    ///
    /// 对应 Java：`DataSignatureInnerInterceptor.verifyResultSet(...)`
    fn verify_result_signatures(&self, value: &mut Value) {
        let sig_col = self.handler.signature_column();
        match value {
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.verify_result_signatures(item);
                }
            }
            Value::Map(map) => {
                let sig_value = map.get(&Value::String(sig_col.to_string()));
                if let Value::String(signature) = sig_value {
                    if !signature.is_empty() {
                        let sign_data = self.build_sign_data(map, sig_col);
                        if !self.handler.verify(&sign_data, signature) {
                            log::warn!(
                                "[DataSignature] 签名验证失败: data={}, signature={}",
                                sign_data,
                                signature
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// 构建签名数据（除签名列外的所有字段值拼接）。
    pub(crate) fn build_sign_data(
        &self,
        map: &rbs::value::map::ValueMap,
        exclude_col: &str,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();
        for (k, v) in map.0.iter() {
            if let Value::String(col_name) = k {
                if col_name != exclude_col {
                    parts.push(format!("{}={}", col_name, value_to_string(v)));
                }
            }
        }
        parts.join("&")
    }
}

/// 将 Value 转为字符串（用于签名计算）。
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::I32(n) => n.to_string(),
        Value::I64(n) => n.to_string(),
        Value::U32(n) => n.to_string(),
        Value::U64(n) => n.to_string(),
        Value::F32(f) => f.to_string(),
        Value::F64(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Binary(b) => format!("{:?}", b),
        _ => format!("{:?}", value),
    }
}

#[async_trait]
impl InnerInterceptor for DataSignatureInnerInterceptor {
    /// INSERT/UPDATE 前计算数据签名。
    ///
    /// 对应 Java：`DataSignatureInnerInterceptor.beforeUpdate(...)`
    async fn before_update(
        &self,
        _executor: &dyn Executor,
        _sql: &mut String,
        args: &mut Vec<Value>,
        _result: &mut Result<ExecResult, Error>,
    ) -> Result<Action, Error> {
        self.sign_args(args);
        Ok(Action::Next)
    }

    /// SELECT 后验证结果集中的签名完整性。
    ///
    /// 对应 Java：`DataSignatureInnerInterceptor.afterQuery(...)`
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        if let Ok(value) = result {
            self.verify_result_signatures(value);
        }
        Ok(())
    }
}

#[async_trait]
impl EnhanceInnerInterceptor for DataSignatureInnerInterceptor {
    /// 声明阶段为数据签名（300）。
    ///
    /// 对应 Java：`DataSignatureInnerInterceptor.phase()`
    fn phase(&self) -> EnhancePhase {
        EnhancePhase::DATA_SIGNATURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::default_handler::DefaultDataSignatureHandler;

    /// 创建测试用拦截器。
    fn create_test_interceptor() -> DataSignatureInnerInterceptor {
        let handler = DefaultDataSignatureHandler::default();
        DataSignatureInnerInterceptor::new(Box::new(handler))
    }

    #[test]
    fn test_phase_is_data_signature() {
        let interceptor = create_test_interceptor();
        assert_eq!(interceptor.phase(), EnhancePhase::DATA_SIGNATURE);
    }

    #[test]
    fn test_sign_args_adds_signature_column() {
        let interceptor = create_test_interceptor();
        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("name".to_string()),
            Value::String("Alice".to_string()),
        );
        map.insert(Value::String("age".to_string()), Value::I32(30));
        let mut args = vec![Value::Map(map)];

        interceptor.sign_args(&mut args);

        if let Value::Map(ref m) = args[0] {
            let sig = m.get(&Value::String("data_signature".to_string()));
            if let Value::String(s) = sig {
                assert!(!s.is_empty(), "签名不应为空");
            } else {
                panic!("签名字段应为 String 类型");
            }
        } else {
            panic!("参数应为 Map 类型");
        }
    }

    #[test]
    fn test_verify_valid_signature() {
        let interceptor = create_test_interceptor();
        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("name".to_string()),
            Value::String("Bob".to_string()),
        );
        let sign_data = interceptor.build_sign_data(&map, "data_signature");
        let signature = interceptor.sign(&sign_data);

        assert!(interceptor.verify(&sign_data, &signature));
    }

    #[test]
    fn test_verify_tampered_data_fails() {
        let interceptor = create_test_interceptor();
        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("name".to_string()),
            Value::String("Bob".to_string()),
        );
        let sign_data = interceptor.build_sign_data(&map, "data_signature");
        let signature = interceptor.sign(&sign_data);

        let tampered_data = sign_data.replace("Bob", "Eve");
        assert!(!interceptor.verify(&tampered_data, &signature));
    }

    #[test]
    fn test_verify_result_signatures_with_valid_data() {
        let interceptor = create_test_interceptor();
        let mut map = rbs::value::map::ValueMap::new();
        map.insert(
            Value::String("name".to_string()),
            Value::String("Charlie".to_string()),
        );
        let sign_data = interceptor.build_sign_data(&map, "data_signature");
        let signature = interceptor.sign(&sign_data);
        map.insert(
            Value::String("data_signature".to_string()),
            Value::String(signature),
        );

        let mut value = Value::Map(map);
        // 验证不应 panic（签名有效）
        interceptor.verify_result_signatures(&mut value);
    }
}
