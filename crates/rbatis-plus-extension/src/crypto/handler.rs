//! 加密字段处理器 trait（对标 Java `EncryptedFieldHandler`）。

/// 单字段加密/解密与 HMAC 运算端口（对标 Java `EncryptedFieldHandler`）。
///
/// 业务可实现该接口接入 KMS、硬件密码机或自定义密钥管理方案。
/// 默认实现使用 XOR + Base64（仅用于演示，生产环境请使用 AES 等强加密）。
///
/// # 对应 Java
///
/// - `com.baomidou.mybatisplus.enhance.crypto.handler.EncryptedFieldHandler`
pub trait EncryptedFieldHandler: Send + Sync + 'static {
    /// 将字段值加密为可持久化字符串。
    ///
    /// 对应 Java `EncryptedFieldHandler.encrypt(T value)`
    fn encrypt(&self, value: &str) -> String;

    /// 解密持久化字符串为目标值。
    ///
    /// 对应 Java `EncryptedFieldHandler.decrypt(String value, Class<T> rtType)`
    fn decrypt(&self, encrypted: &str) -> String;

    /// 计算字段值的 HMAC 签名。
    ///
    /// 对应 Java `EncryptedFieldHandler.hmac(T value)`
    fn hmac(&self, value: &str) -> String;

    /// 验证 HMAC 签名（默认实现：直接比较）。
    ///
    /// 对应 Java `EncryptedFieldHandler.verifyHmac(T value, String signature)`
    fn verify_hmac(&self, value: &str, signature: &str) -> bool {
        self.hmac(value) == signature
    }

    /// 判断字段是否需要加密（根据列名）。
    ///
    /// 对应 Java 中通过 `@EncryptedField` 注解判断
    fn should_encrypt(&self, _column: &str) -> bool {
        false // 默认不加密，由拦截器根据注解配置决定
    }
}
