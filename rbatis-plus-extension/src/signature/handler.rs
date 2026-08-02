//! 数据签名处理器 trait（对标 Java `DataSignatureHandler`）。

/// 数据签名处理器（对标 Java `DataSignatureHandler`）。
///
/// 在 INSERT/UPDATE 时计算数据签名，在 SELECT 后验证签名完整性。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.crypto.handler.DataSignatureHandler`
pub trait DataSignatureHandler: Send + Sync + 'static {
    /// 计算数据签名。
    ///
    /// 对应 Java `DataSignatureHandler.sign(Object data)`
    fn sign(&self, data: &str) -> String;

    /// 验证数据签名。
    ///
    /// 对应 Java `DataSignatureHandler.verify(Object data, String signature)`
    fn verify(&self, data: &str, signature: &str) -> bool;

    /// 签名列名（默认 "data_signature"）。
    fn signature_column(&self) -> &str {
        "data_signature"
    }
}
