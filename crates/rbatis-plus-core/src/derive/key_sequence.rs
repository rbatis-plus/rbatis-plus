/// 序列主键策略（Oracle 等数据库使用序列生成 ID）。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.KeySequence`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/KeySequence.java`
///
/// 用于在执行 MyBatis-Plus 方法前通过数据库序列获取主键值。
///
/// ```rust
/// use rbatis_plus_core::KeySequence;
///
/// trait UserKeySequence {
///     // derive 宏自动实现
/// }
///
/// // derive(KeySequence) 生成的实现示例：
/// // impl KeySequence for User {
/// //     fn sequence_name() -> &'static str { "SEQ_USER_ID" }
/// //     fn db_type() -> &'static str { "ORACLE" }
/// // }
/// ```
pub trait KeySequence {
    /// 序列名（对应 Java `@KeySequence.value()`；默认空串表示不使用序列）。
    fn sequence_name() -> &'static str {
        ""
    }

    /// 数据库类型（对应 Java `@KeySequence.dbType()`；默认 "OTHER"）。
    /// 多个实现时必须指定，让框架知道使用哪个 KeyGenerator。
    fn db_type() -> &'static str {
        "OTHER"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty_string() {
        struct MockKeySeq;
        impl KeySequence for MockKeySeq {}

        assert_eq!(MockKeySeq::sequence_name(), "");
        assert_eq!(MockKeySeq::db_type(), "OTHER");
    }

    #[test]
    fn custom_sequence() {
        struct MockOracleSeq;
        impl KeySequence for MockOracleSeq {
            fn sequence_name() -> &'static str { "SEQ_USER_ID" }
            fn db_type() -> &'static str { "ORACLE" }
        }

        assert_eq!(MockOracleSeq::sequence_name(), "SEQ_USER_ID");
        assert_eq!(MockOracleSeq::db_type(), "ORACLE");
    }
}
