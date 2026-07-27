/// 字段自动排序标记。
///
/// 对应 Java：`com.baomidou.mybatisplus.annotation.OrderBy`（mybatis-plus-annotation）
/// 文件来源参考：`mybatis-plus-annotation/src/main/java/com/baomidou/mybatisplus/annotation/OrderBy.java`
///
/// 在执行 MyBatis-Plus 的方法 `selectList()`、`Page()` 等非手写查询时自动带上 ORDER BY。
/// 用法与 Spring Data JPA 的 `@OrderBy` 类似。
///
/// ```rust
/// use rbatis_plus_core::OrderBy;
///
/// trait UserOrderBy {
///     // derive 宏自动实现
/// }
///
/// // derive(OrderBy) 生成的实现示例：
/// // impl OrderBy for User {
/// //     fn order_by_asc() -> bool { false }     // 默认倒序
/// //     fn order_by_sort() -> u16 { 32767 }    // 默认 Short.MAX_VALUE
/// // }
/// ```
pub trait OrderBy {
    /// 是否正序（对应 Java `@OrderBy.asc()`；默认 false → 倒序）。
    fn order_by_asc() -> bool {
        false
    }

    /// 排序优先级（对应 Java `@OrderBy.sort()`；默认 32767 = Short.MAX_VALUE，数字越小越靠前）。
    fn order_by_sort() -> u16 {
        u16::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_desc_and_low_priority() {
        struct MockOrder;
        impl OrderBy for MockOrder {}

        assert!(!MockOrder::order_by_asc());
        assert_eq!(MockOrder::order_by_sort(), u16::MAX);
    }

    #[test]
    fn custom_asc_and_sort() {
        struct MockAscOrder;
        impl OrderBy for MockAscOrder {
            fn order_by_asc() -> bool { true }
            fn order_by_sort() -> u16 { 10 }
        }

        assert!(MockAscOrder::order_by_asc());
        assert_eq!(MockAscOrder::order_by_sort(), 10);
    }
}
