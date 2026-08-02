//! 默认数据签名处理器（对标 Java `DefaultDataSignatureHandler`）。
//!
//! 使用 XOR + 十六进制编码实现，仅用于演示。
//! 生产环境请实现 `DataSignatureHandler` trait 接入 HMAC-SHA256 等强签名算法。

use super::handler::DataSignatureHandler;

/// 默认数据签名处理器（对标 Java `DefaultDataSignatureHandler`）。
///
/// **安全警告**：此实现仅用于演示，不提供真实安全性。
#[derive(Debug, Clone)]
pub struct DefaultDataSignatureHandler {
    /// 签名密钥。
    key: Vec<u8>,
    /// 签名列名。
    signature_column: String,
}

impl Default for DefaultDataSignatureHandler {
    fn default() -> Self {
        Self {
            key: b"rbatis-plus-signature".to_vec(),
            signature_column: "data_signature".to_string(),
        }
    }
}

impl DefaultDataSignatureHandler {
    /// 使用自定义密钥创建。
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
            signature_column: "data_signature".to_string(),
        }
    }

    /// 设置签名列名。
    pub fn with_signature_column(mut self, column: impl Into<String>) -> Self {
        self.signature_column = column.into();
        self
    }
}

impl DataSignatureHandler for DefaultDataSignatureHandler {
    /// 计算签名：XOR + 十六进制编码。
    fn sign(&self, data: &str) -> String {
        let bytes = data.as_bytes();
        let xored: Vec<u8> = bytes
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.key[i % self.key.len()])
            .collect();
        xored.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// 验证签名：重新计算并比较。
    fn verify(&self, data: &str, signature: &str) -> bool {
        self.sign(data) == signature
    }

    /// 签名列名。
    fn signature_column(&self) -> &str {
        &self.signature_column
    }
}
