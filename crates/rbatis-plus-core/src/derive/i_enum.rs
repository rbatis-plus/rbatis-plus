/// 自定义枚举接口。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.IEnum<T extends Serializable>`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/IEnum.java`
///
/// 枚举类型可以实现此 trait，在 MyBatis-Plus 的 TypeHandler 中自动序列化为数据库值。
///
/// ```rust
/// use rbatis_plus_core::IEnum;
///
/// #[derive(Debug)]
/// enum Status {
///     Active,
///     Inactive,
/// }
///
/// impl IEnum<u8> for Status {
///     fn value(&self) -> u8 {
///         match self {
///             Self::Active => 1,
///             Self::Inactive => 0,
///         }
///     }
/// }
///
/// assert_eq!(Status::Active.value(), 1u8);
/// ```
///
/// **设计说明**：Java 版泛型参数是 `<T extends Serializable>`（Java 序列化链路），
/// Rust 版泛型参数 `<T>` 仅要求 `PartialEq` + `Clone` + `Send` + `Sync`（运行时比较/克隆/并发安全），
/// 因为 Rust 没有 Java Serializable 接口。
pub trait IEnum<T: PartialEq + Clone + Send + Sync> {
    /// 枚举数据库存储值（对应 Java `IEnum.getValue()`）。
    fn value(&self) -> T;
}

/// 根据枚举值从数据库反序列化（Rust 手动实现，无需 Java 的 `AbstractEnumTypeHandler.getValue()`）。
///
/// 提供通用的反序列化辅助函数：给定一个 `values` 迭代器和目标值，找到匹配的枚举项。
pub fn from_value<'a, T, E>(values: &'a [E], target: &T) -> Option<&'a E>
where
    T: PartialEq + Clone + Send + Sync,
    E: IEnum<T>,
{
    values.iter().find(|e| e.value() == *target)
}

/// 枚举值类型 marker（对应 Java Serializable 语义的 Rust 型别化表示）。
///
/// 在 Rust 中没有真正的 Java Serializable 等价物，这里用 marker trait
/// 限定 `IEnum` 的 `T` 必须是可序列化的简单类型（不是 trait object）。
pub trait EnumValueType: PartialEq + Clone + Send + Sync + std::fmt::Debug + 'static {}
impl EnumValueType for i8 {}
impl EnumValueType for i16 {}
impl EnumValueType for i32 {}
impl EnumValueType for i64 {}
impl EnumValueType for u8 {}
impl EnumValueType for u16 {}
impl EnumValueType for u32 {}
impl EnumValueType for u64 {}
impl EnumValueType for String {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    enum UserStatus {
        Active = 1,
        Inactive = 2,
    }

    impl IEnum<i32> for UserStatus {
        fn value(&self) -> i32 {
            match self {
                Self::Active => 1,
                Self::Inactive => 2,
            }
        }
    }

    #[test]
    fn value_matches_java_semantics() {
        assert_eq!(UserStatus::Active.value(), 1);
        assert_eq!(UserStatus::Inactive.value(), 2);
    }

    #[test]
    fn from_value_finds_match() {
        let values = [UserStatus::Active, UserStatus::Inactive];
        assert_eq!(from_value(&values, &1i32), Some(&UserStatus::Active));
        assert_eq!(from_value(&values, &2i32), Some(&UserStatus::Inactive));
        assert_eq!(from_value(&values, &99i32), None);
    }

    #[test]
    fn java_ienum_semantics() {
        // Java IEnum<T> 的核心契约：
        // 1. 枚举实现接口 getValue() → T
        // 2. TypeHandler 调用 getValue() 序列化为数据库值
        // 3. TypeHandler 根据数据库值反序列化为枚举
        // Rust 端完全对齐此行为
        let status = UserStatus::Active;
        let db_value: i32 = status.value();
        assert_eq!(db_value, 1);
    }
}
