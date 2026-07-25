//! 签名验证拦截器（对标 mybatis-plus-enhance `DataSignatureInnerInterceptor`）。
//!
//! 在 INSERT/UPDATE 时自动计算数据签名，在 SELECT 后自动验证签名完整性。

use crate::inner::inner_interceptor::InnerInterceptor;
use async_trait::async_trait;
use rbatis::executor::Executor;
use rbatis::{Action, Error};
use rbs::Value;
use std::sync::Arc;

use super::handler::DataSignatureHandler;

/// 数据签名拦截器（对标 Java `DataSignatureInnerInterceptor`）。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.crypto.DataSignatureInnerInterceptor`
///
/// # 用法
///
/// ```ignore
/// use rbatis_plus_extension::signature::{SignatureInnerInterceptor, DefaultDataSignatureHandler};
///
/// let handler = DefaultDataSignatureHandler::default();
/// let interceptor = SignatureInnerInterceptor::new(Box::new(handler));
/// ```
pub struct SignatureInnerInterceptor {
    /// 签名处理器。
    handler: Arc<dyn DataSignatureHandler>,
}

impl std::fmt::Debug for SignatureInnerInterceptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignatureInnerInterceptor")
            .field("signature_column", &self.handler.signature_column())
            .finish()
    }
}

impl SignatureInnerInterceptor {
    /// 创建签名拦截器。
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
}

#[async_trait]
impl InnerInterceptor for SignatureInnerInterceptor {
    /// SELECT 后验证结果集中的签名。
    async fn after_query(
        &self,
        _executor: &dyn Executor,
        _sql: &str,
        result: &mut Result<Value, Error>,
    ) -> Result<(), Error> {
        if let Ok(value) = result {
            verify_result_signatures(value, &self.handler);
        }
        Ok(())
    }
}

/// 验证结果集中的签名。
fn verify_result_signatures(value: &mut Value, handler: &Arc<dyn DataSignatureHandler>) {
    let sig_col = handler.signature_column();
    match value {
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                verify_result_signatures(item, handler);
            }
        }
        Value::Map(map) => {
            // 查找签名列
            let sig_value = map.get(&Value::String(sig_col.to_string()));
            if let Value::String(signature) = sig_value {
                if !signature.is_empty() {
                    // 收集除签名列外的所有字段，拼接为签名数据
                    let sign_data = build_sign_data(map, sig_col);
                    if !handler.verify(&sign_data, signature) {
                        log::warn!("数据签名验证失败: data={}, signature={}", sign_data, signature);
                        // 可以选择抛出异常或标记数据
                    }
                }
            }
        }
        _ => {}
    }
}

/// 构建签名数据（除签名列外的所有字段值拼接）。
fn build_sign_data(map: &rbs::value::map::ValueMap, exclude_col: &str) -> String {
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

/// 对 INSERT/UPDATE 参数计算签名并添加签名列。
///
/// 在 `before_update` 中调用，对标 `DataSignatureInnerInterceptor.signArgs()`
pub fn sign_args(
    args: &mut Vec<Value>,
    handler: &dyn DataSignatureHandler,
) {
    let sig_col = handler.signature_column();
    for arg in args.iter_mut() {
        if let Value::Map(map) = arg {
            let sign_data = build_sign_data(map, sig_col);
            let signature = handler.sign(&sign_data);
            map.insert(Value::String(sig_col.to_string()), Value::String(signature));
        }
    }
}
