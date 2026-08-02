//! 增强拦截器阶段顺序枚举（对齐 Java `mybatis-plus-enhance` 的 `EnhancePhase`）。
//!
//! 阶段顺序约束：
//! - 写入前处理：参数先加密再签名
//! - 查询后处理：先验签再解密，解密后才能执行国际化，观测最后执行
//!
//! 未声明阶段的自定义增强不参与强制排序（UNSPECIFIED）。

/// 增强拦截器阶段。
///
/// 对应 Java：`com.baomidou.mybatisplus.enhance.plugins.inner.EnhancePhase`
/// 文件来源参考：`mybatis-plus-enhance-core/src/main/java/com/baomidou/mybatisplus/enhance/plugins/inner/EnhancePhase.java`
///
/// 数值越小越先执行。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)] // 枚举名与 Java EnhancePhase 一一对应（如 SQL_REWRITE、RESULT_I18N）
pub enum EnhancePhase {
    /// SQL 结构改写或前置保护（如 INSERT IGNORE、超长 SQL 检测）。
    SQL_REWRITE = 100,
    /// 写入参数加密（DataEncryptionInnerInterceptor）。
    PARAMETER_ENCRYPTION = 200,
    /// 写入签名及查询结果验签（DataSignatureInnerInterceptor）。
    DATA_SIGNATURE = 300,
    /// 查询结果解密（DataDecryptionInnerInterceptor）。
    RESULT_DECRYPTION = 400,
    /// 查询结果国际化（DataI18nInnerInterceptor）。
    RESULT_I18N = 500,
    /// SQL 执行观测与旁路通知（SqlObservationInnerInterceptor）。
    OBSERVATION = 900,
    /// 不参与框架顺序校验的自定义阶段（第三方增强默认）。
    UNSPECIFIED = -10000,
}

impl EnhancePhase {
    /// 获取阶段排序值（数值越小越先执行）。
    pub fn order(self) -> i32 {
        self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_ordering() {
        assert!(EnhancePhase::SQL_REWRITE.order() < EnhancePhase::PARAMETER_ENCRYPTION.order());
        assert!(EnhancePhase::PARAMETER_ENCRYPTION.order() < EnhancePhase::DATA_SIGNATURE.order());
        assert!(EnhancePhase::DATA_SIGNATURE.order() < EnhancePhase::RESULT_DECRYPTION.order());
        assert!(EnhancePhase::RESULT_DECRYPTION.order() < EnhancePhase::RESULT_I18N.order());
        assert!(EnhancePhase::RESULT_I18N.order() < EnhancePhase::OBSERVATION.order());
        assert!(EnhancePhase::UNSPECIFIED.order() < EnhancePhase::SQL_REWRITE.order());
    }
}
